const express = require("express");
const db = require("../db.js");
const fs = require("fs");
const crypto = require("crypto");

const router = express.Router();

const requiresLogin = (req, res, next) =>
  req.user
    ? next()
    : res.json({ success: false, error: "You must be logged in!" });

router.post("/login", (req, res) => {
  let { user, pass } = req.body;
  if (!user || !pass || typeof user !== "string" || typeof pass !== "string") {
    return res.json({
      success: false,
      error: "Missing username or password",
    });
  }

  if (!db.hasUser({user: user})) {
    return res.json({
      success: false,
      error: "No user exists with that username",
    });
  }

  if (!db.checkPass({user: user, pass: pass})) {
    return res.json({ success: false, error: "Invalid password" });
  }

  req.session.user = user;
  res.json({ success: true });
});

router.post("/register", (req, res) => {
  let { user, pass } = req.body;
  if ( !user || !pass || typeof user !== "string" || typeof pass !== "string") {
    return res.json({
      success: false,
      error: "Missing username or password",
    });
  }

  if (db.hasUser({user: user})) {
    return res.json({
      success: false,
      error: "User already exists",
    });
  }

  req.session.user = user;
	db.addUser({user: user, pass: pass});
  res.json({ success: true });
});

router.post("/create", requiresLogin, (req, res) => {
  let { feed, banned_phrases } = req.body;
  if (!feed || !banned_phrases || typeof feed !== "string" || typeof banned_phrases !== "string") {
    return res.json({ success: false, error: "Missing name or phrases" });
  }
  
  try {
    const parsed = JSON.parse(banned_phrases);
    if (!Array.isArray(parsed)) {
      return res.json({ success: false, error: "Must supply phrases as a JSON-encoded array" });
    }
    
    if (parsed.length > 100) {
      return res.json({ success: false, error: "Feed cannot have more than 100 phrases" });
    }
    
    db.addFeed({user: req.user.username, feed: feed, banned_phrases: parsed});
    return res.json({ success: true });
  } catch (error) {
    return res.json({ success: false, error: "Must supply phrases as a JSON-encoded array" });
  }  
});

router.post("/update", requiresLogin, (req, res) => {
  let { feed, banned_phrases } = req.body;
  if (!banned_phrases || typeof banned_phrases !== "string") {
    return res.json({ success: false, error: "Missing phrases" });
  }
  
  if (req.user.username !== db.getFeed({feed: feed}).owner) {
    return res.json({ success: false, error: "Only owner can update feed" });
  }
  
  try {
    const parsed = JSON.parse(banned_phrases);
    if (!Array.isArray(parsed)) {
      return res.json({ success: false, error: "Must supply phrases as a JSON-encoded array" });
    }
    
    if (parsed.length > 100) {
      return res.json({ success: false, error: "Feed cannot have more than 100 phrases" });
    }
    
    db.updateFeed({feed: feed, banned_phrases: parsed});
    return res.json({ success: true });
  } catch (error) {
    return res.json({ success: false, error: "Must supply phrases as a JSON-encoded array" });
  }  
});

router.post("/subscribe", requiresLogin, (req, res) => {
  let { feed } = req.body;
  if (!feed || typeof feed !== "string") {
    return res.json({ success: false, error: "Missing feed" });
  }
  
  if (req.user.subscriptions.includes(feed)) {
    return res.json({ success: false, error: "Already subscribed to feed!" });
  }
  
  db.addSubscription({user: req.user.username, feed: feed});
  return res.json({ success: true });
});

router.post("/unsubscribe", requiresLogin, (req, res) => {
  let { feed } = req.body;
  if (!feed || typeof feed !== "string") {
    return res.json({ success: false, error: "Missing feed" });
  }
  
  if (!req.user.subscriptions.includes(feed)) {
    return res.json({ success: false, error: "Not subscribed to feed!" });
  }
  
  db.removeSubscription({user: req.user.username, feed: feed});
  return res.json({ success: true });
});

router.post("/feeds", requiresLogin, (req, res) => {
  return res.json({
    success: true,
    data: [...db.feeds.keys()],
  });
});

module.exports = router;
