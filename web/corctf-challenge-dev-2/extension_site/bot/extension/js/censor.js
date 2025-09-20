// taken from https://stackoverflow.com/a/50537862
// this will still miss cases like "CENSOR <strong>THIS</strong>"
// but let's be real nobody actually believes this extension is really supposed to censor anything
// and im not importing findAndReplaceDOMText for a ctf challenge
function replaceInText(element, pattern, replacement) {
    for (let node of element.childNodes) {
        switch (node.nodeType) {
            case Node.ELEMENT_NODE:
                replaceInText(node, pattern, replacement);
                break;
            case Node.TEXT_NODE:
                node.textContent = node.textContent.replaceAll(pattern, replacement);
                break;
            case Node.DOCUMENT_NODE:
                replaceInText(node, pattern, replacement);
        }
    }
}

(async () => {
  const response = await chrome.runtime.sendMessage({type: "get_banned"});
  const banned_phrases = response.banned_phrases;
  // "oh but couldnt you implement aho-corasick instead of naively looping indexOf"
  // i am a javascript dev you are lucky this algorithm doesnt run in O(n!) time
  for (const phrase of banned_phrases) {
    // only try replacing if there actually is anything to replace
    let found = (document.documentElement.textContent || document.documentElement.innerText).indexOf(phrase);
    if (found > -1) {
      replaceInText(document, phrase, "[CENSORED]");
    }
  }
})();

// inject resync button
const resync = document.createElement('button');
resync.type = 'button';
resync.id = 'resync';
resync.textContent = "Resync block settings";

// resync listener
resync.addEventListener('click', async () => {
  resync.textContent = "Resyncing block settings...";
  const response = await chrome.runtime.sendMessage({type: "resync"});
  resync.textContent = "Synced!";
});

document.body.insertBefore(resync, document.body.childNodes[0]);