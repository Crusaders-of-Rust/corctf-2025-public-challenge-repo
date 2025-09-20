const crypto = require("crypto");

const users = new Map();
const feeds = new Map();

const connected_clients = new Map();

const sha256 = (data) => crypto.createHash("sha256").update(data).digest("hex");

const addUser = ({ user, pass }) => {
	users.set(user, {
    username: user, // ok i know this is stupid but i really cant think of a better way and i havent slept in 48 hours so into prod it goes
		pass: sha256(pass),
		subscriptions: [],
    last_updated: Date.now()
	});
};

const hasUser = ({ user }) => {
	return users.has(user);
};

const getUser = ({ user }) => {
	return users.get(user);
};

const checkPass = ({ user, pass }) => {
	return users.get(user).pass === sha256(pass)
};

const addFeed = ({ user, feed, banned_phrases }) => {
  feeds.set(feed, {
    banned_phrases: banned_phrases,
    subscribers: [],
    owner: user
  });
};

const hasFeed = ({ feed }) => {
  return feeds.has(feed);
}

const getFeed = ({ feed }) => {
  return feeds.get(feed);
};

const updateFeed = ({ feed, banned_phrases }) => {
  feeds.get(feed).banned_phrases = banned_phrases;
  feeds.get(feed).subscribers.forEach((subscriber) => {
    const u = users.get(subscriber);
    u.last_updated = Date.now();
    
    if (connected_clients.has(subscriber)) {
      const banned_phrases = []; // TODO: add code here to pick between "update" and "add"/"remove" depending on how large the feed is
      u.subscriptions.forEach((feed) => {
        banned_phrases.push(...feeds.get(feed).banned_phrases);
      });
      connected_clients.get(subscriber).forEach((socket) => {
        socket.send(JSON.stringify({type: "update", banned_phrases: banned_phrases}));
      });
    }
  });
};

const addSubscription = ({ user, feed }) => {
  users.get(user).subscriptions.push(feed);
  feeds.get(feed).subscribers.push(user);
  
  users.get(user).last_updated = Date.now();
  
  if (connected_clients.has(user)) {
    connected_clients.get(user).forEach((socket) => {
      socket.send(JSON.stringify({type: "add", banned_phrases: feeds.get(feed).banned_phrases}));
    });
  }
};

const removeSubscription = ({ user, feed }) => {
  const u = users.get(user).subscriptions;
  const f = feeds.get(feed).subscribers;
  
  u.splice(u.indexOf(feed), 1);
  f.splice(f.indexOf(user), 1);
  
  users.get(user).last_updated = Date.now();
  
  if (connected_clients.has(user)) {
    connected_clients.get(user).forEach((socket) => {
      socket.send(JSON.stringify({type: "remove", banned_phrases: feeds.get(feed).banned_phrases}));
    });
  }
};

const addConnectedClient = ({ user, socket }) => {
  if (!connected_clients.has(user)) {
    connected_clients.set(user, []);
  }
  connected_clients.get(user).push(socket);
}

const removeConnectedClient = ({ socket }) => {
  const c = connected_clients.get(socket.name);
  c.splice(c.indexOf(socket), 1);
}

addUser({user: "fizzbuzz101", pass: process.env.ADMIN_PASSWORD || "test_password"});
addFeed({user: "fizzbuzz101", feed: "pwn", banned_phrases: ["starlabs", "0click", "dup", "cross-cache", "100k typo"]}); // TODO: approved by fizzbuzz101 himself :')
addSubscription({user: "fizzbuzz101", feed: "pwn"});

module.exports = { users, feeds, connected_clients, addUser, hasUser, getUser, checkPass, addFeed, hasFeed, getFeed, updateFeed, addSubscription, removeSubscription, addConnectedClient, removeConnectedClient };
