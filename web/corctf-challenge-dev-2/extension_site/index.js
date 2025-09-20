const ws = require("ws");

const express = require("express");
const crypto = require("crypto");
const session = require("express-session");
const MemoryStore = require("memorystore")(session);

const app = express();
const PORT = process.env.PORT || 3000;

const db = require("./db.js");
const bot = require("./bot/bot.js");

app.use(
    session({
        cookie: { maxAge: 3600000 },
        store: new MemoryStore({
            checkPeriod: 3600000,
        }),
        resave: false,
        saveUninitialized: false,
        secret: crypto.randomBytes(32).toString("hex"),
    })
);

app.use(express.static("public"));
app.use(express.json());
app.use(express.urlencoded({ extended: true }));

app.use((req, res, next) => {
	const nonce = crypto.randomBytes(16).toString('base64');
  res.setHeader(
    "Content-Security-Policy",
    `base-uri 'none'; script-src 'nonce-${nonce}';`
  );

  res.locals.user = null;
  if (req.session.user && db.hasUser({user: req.session.user})) {
    req.user = db.getUser({user: req.session.user});
    res.locals.user = req.user;
	}
  
	res.locals.nonce = nonce;
  next();
});

app.set('view engine', 'ejs');

app.use("/api", require("./routes/api.js"));

const requiresLogin = (req, res, next) => 
	req.user
		? next()
		: res.redirect('/login');

app.get("/", (req, res) => res.render("index"));

app.get("/login", (req, res) => res.render("login"));

app.get("/register", (req, res) => res.render("register"));

app.get("/create", requiresLogin, (req, res) => res.render("create"));

app.get("/edit/:name", requiresLogin, (req, res) => {
  let { name } = req.params;
  if (!name) {
    return res.json({ success: false, error: "No feed name provided" });
  }
  if (!db.hasFeed({feed: name})) {
    return res.status(404).send("Feed not found!");
  }
  feed = db.getFeed({feed: name});
  if (req.user.username !== feed.owner) {
    return res.status(403).send("Not feed owner!");
  }
  
  res.render('edit', { name, feed });
});

app.get("/feeds", requiresLogin, (req, res) => res.render("feeds"));

app.get("/feed/:name", (req, res) => {
  let { name } = req.params;
  if (!name) {
    return res.json({ success: false, error: "No feed name provided" });
  }
  if (!db.hasFeed({feed: name})) {
    return res.status(404).send("Feed not found!");
  }
	feed = db.getFeed({feed: name});
	res.render('feed', { name, feed });
});

app.get("/submit", requiresLogin, (req, res) => {
  // submit both the url of the instanced site (e.g. https://fizzblock-101-deadbeef.ctfi.ng) and your url to visit
  // this is an infra workaround. you are NOT meant to try to bypass checks and submit an attacker-controlled url for instance_url
  const { url } = req.query;
    
  if (!url || typeof url !== "string") {
    return res.send('missing url');
  }

  const urlObj = new URL(url);
  if (!['http:', 'https:'].includes(urlObj.protocol)) {
    return res.send('url must be http/https')
  }
  
  bot.visit(url);
  res.send('the admin will visit your url soon');
});

const server = app.listen(PORT, () => console.log(`app listening on port ${PORT}`));

const wss = new ws.Server({ server: server, path: '/ws' });

wss.on('connection', (socket) => {  
  socket.send("ready");
  socket.state = 0; // 0 = waiting for name, 1 = waiting for password, 2 = waiting for sync decision, 3 = receiving feed updates
  
  socket.on('message', (message) => {
    if (message.toString() == "keepalive") { // discard keepalives
      return;
    }
    
    if (socket.state === 0) { // received username, challenge for password    
      if (!db.hasUser({user: message.toString()})) { // unknown user
        socket.close();
        return;
      }
      socket.name = message.toString();
      socket.send("ok");
      
      socket.state++;
    }
    else if (socket.state === 1) { // check password, update connected clients if valid
      if (!db.checkPass({user: socket.name, pass: message.toString()})) {
        socket.close();
        return;
      }
      socket.user = db.getUser({user: socket.name});
      db.addConnectedClient({user: socket.name, socket: socket});
      
      // to avoid unnecessarily sending traffic, only update the client if their information is out of date
      socket.send(socket.user.last_updated);
      
      socket.state++;
    }
    else if (socket.state === 2) { // receive sync decision
      if (message.toString() === "update") {
        const banned_phrases = [];
        socket.user.subscriptions.forEach((feed) => {
          banned_phrases.push(...db.getFeed({feed: feed}).banned_phrases);
        });
        socket.send(JSON.stringify({type: "update", banned_phrases: banned_phrases}));
      }
      socket.state++;
    }
    else { // feed updates
      try {
        const parsed = JSON.parse(message.toString());
        if (parsed.type == "update") {
          if (!parsed.feed || !parsed.banned_phrases || !db.hasFeed({feed: parsed.feed}) || !Array.isArray(parsed.banned_phrases)) {
            socket.send("missing feed or phrases");
            return;
          }
          
          if (socket.name !== db.getFeed({feed: parsed.feed}).owner) {
            socket.send("cannot edit feed that you don't own");
            return;
          }
          
          db.updateFeed({feed: parsed.feed, banned_phrases: parsed.banned_phrases});
          socket.send("ok");
        }
      } catch (error) {
        socket.send("malformed data");
        return;
      }
    }
  });
  
  socket.on('close', () => {
    if (socket.state > 1) { // connected client added
      db.removeConnectedClient({socket: socket});
    }
  });
});