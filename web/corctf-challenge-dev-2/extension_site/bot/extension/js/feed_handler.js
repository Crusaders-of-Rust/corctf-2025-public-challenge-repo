// serverUrl is supposed to point at a remote resource -- we're using localhost here due to infra limitations
// if you want to test this extension locally, you should set serverUrl = "wss://fizzblock-101-deadbeef.ctfi.ng/ws" to point at your instanced server
const serverUrl = "ws://localhost:3000/ws"; // INSTANCER WILL OVERWRITE "PORT" WITH THE CONFIGURED PORT

let socket;
let last_updated = 0;
let banned_phrases = [];

function connect() {
  socket = new WebSocket(serverUrl);
  
  socket.onopen = (event) => {
    keepAlive();
  };
  
  socket.state = 0; // 0 = send name, 1 = decide sync, 2 = receive sync, >3 = receiving updates (at this point, state counts number of updates received)

  socket.onmessage = (event) => {
    if (socket.state === 0) { // should we check event.data === "ready"? it's probably fine, right
      socket.send("fizzbuzz101");
      socket.state++;
    }
    else if (socket.state === 1) { // send password
      socket.send("18af8d6f5a98ddbe55d16540f174bc16"); // INSTANCER WILL OVERWRITE "ADMIN_PASSWORD" WITH THE REAL ADMIN PASSWORD
      socket.state++;
    }
    else if (socket.state === 2) { // check timestamp
      if (event.data > last_updated) { // newer information available, request update
        socket.send("update");
      }
      else {
        socket.send("no update");
      }
      socket.state++;
    }
    else { // update
      const update = JSON.parse(event.data);
      if (update.type == "add") {
        banned_phrases.push(...update.banned_phrases);
      }
      else if (update.type == "remove") {
        update.banned_phrases.forEach((phrase) => {banned_phrases.splice(banned_phrases.indexOf(phrase), 1)});
      }
      else if (update.type == "update") {
        banned_phrases = update.banned_phrases;
      }
      else {
        return;
      }
      socket.state++;
    }
  };

  socket.onclose = (event) => {
    socket = null;
  };

  socket.onerror = (err) => {
    console.error(`Error: ${err}`);
  };
}

async function disconnect() {
  if (socket == null) {
    return;
  }
  const p = new Promise(r => socket.onclose = () => { socket = null; r() });
  socket.close();
  return p;
}

function keepAlive() {
  const keepAliveInstance = setInterval(
    () => {
      if (socket) {
        socket.send('keepalive');
      } else {
        clearInterval(keepAliveInstance);
      }
    },
    20 * 1000 
  );
}

connect();

const handle_resync = async (sendResponse) => {
  await disconnect();
  connect();
  await new Promise((resolve) => {
    const checkState = () => {
      if (socket.state >= 4) {
        resolve();
      } else {
        setTimeout(checkState, 100);
      }
    };
    checkState();
  });
  sendResponse({reconnected:true});
}

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  // in theory we should check sender.tab to make sure its coming from a content script, but surely this isnt a vulnerability :clueless:
  if (request.type === "resync") {
    handle_resync(sendResponse);
  }
  else {
    sendResponse({banned_phrases: banned_phrases});
  }
  return true;
});
