// const fs = require("fs");
// const path = require("path");  
const { LRUCache } = require("lru-cache");
const { s3, getMime } = require("./r2Config.js");

const BUCKET = process.env.R2_BUCKET;

// Cache for presigned URLs
const cache = new LRUCache({
  max: 1000,
  ttl: 1000 * 60 * 60, // 1 hour
});

// async function uploadFileToR2(filePath, key, mime) {
//   const ContentType = getMime(key) || mime || "application/octet-stream";
//   const Body = fs.createReadStream(filePath);
//   try {
//     await s3.upload({ Bucket: BUCKET, Key: key, Body, ContentType }).promise();
//   } catch (err) {
//     console.error("R2 upload failed", {
//       endpoint: process.env.R2_ENDPOINT,
//       bucket: BUCKET,
//       key,
//       code: err.code,
//       statusCode: err.statusCode,
//     });
//     throw err;
//   }
// }

async function uploadBufferToR2(buffer, key, mime) {
  const ContentType = getMime(key) || mime || "application/octet-stream";
  const Body = Buffer.isBuffer(buffer) ? buffer : Buffer.from(buffer);
  try {
    await s3.upload({ Bucket: BUCKET, Key: key, Body, ContentType }).promise();
  } catch (err) {
    console.error("R2 upload (buffer) failed", {
      endpoint: process.env.R2_ENDPOINT,
      bucket: BUCKET,
      key,
      code: err.code,
      statusCode: err.statusCode,
    });
    throw err;
  }
}

// function getSignedUrl(key) {
//   if (!key) return null;
//   const base = (process.env.GET_R2_ENDPOINT || "").replace(/\/+$/, "");
//   const cleanedKey = (key + "").replace(/^\/+/, "");
//   const finalKey = cleanedKey.includes("ntpl_dod")
//     ? cleanedKey
//     : `ntpl_dod/assets/${cleanedKey}`;
//   return `${base}/${finalKey}`;
// }
function getSignedUrl(key) {
  const cached = cache.get(key);
  if (cached) return cached;

  if (key.includes("http")) {
    return key;
  }

  const prefix = process.env.R2_KEY_PREFIX;
  if (!key.includes(prefix)) {
    key = `${prefix}/assets/${key}`;
  }

  const signedUrl = s3.getSignedUrl("getObject", {
    Bucket: process.env.R2_BUCKET,
    Key: key,
    Expires: 60 * 60, // 1 hour
  });
  cache.set(key, signedUrl);
  return signedUrl;
}

// Get or cache presigned URL
async function getNonCachedSignedUrl(key, expiry = 3600) {
  if (key.includes("http")) {
    return key;
  }

  const prefix = process.env.R2_KEY_PREFIX;
  if (!key.includes(prefix)) {
    key = `${prefix}/assets/${key}`;
  }

  const signedUrl = s3.getSignedUrl("getObject", {
    Bucket: process.env.R2_BUCKET,
    Key: key,
    Expires: 60 * 60, // 1 hour
  });
  return signedUrl;
}

function extractKeyFromUrl(input) {
  try {
    const u = new URL(input);
    const parts = u.pathname.split("/").filter(Boolean);
    const bucketIndex = parts.findIndex((p) => p === BUCKET);
    if (bucketIndex !== -1) return parts.slice(bucketIndex + 1).join("/");
    const known = [
      "assets",
      "images",
      "videos",
      "doctors",
      "patients",
      "serviceProvider",
      "gallery",
      "events",
      "users",
    ];
    const fi = parts.findIndex((p) => known.includes(p));
    if (fi !== -1) return parts.slice(fi).join("/");
    const final = parts.slice(-3).join("/");
    console.log("final", final);
    return final;
  } catch (_) {
    return null;
  }
}

function normalizeToR2Key(value) {
  if (!value) return null;
  if (typeof value === "string") {
    const s = value
      .trim()
      .replace(/^`+|`+$/g, "")
      .replace(/^"+|"+$/g, "")
      .replace(/^'+|'+$/g, "");
    if (s.startsWith("http")) return extractKeyFromUrl(s);
    return s;
  }
  return null;
}

async function deleteObjectByUrlOrKey(input) {
  let key = input;
  if (typeof input === "string" && input.startsWith("http")) {
    const fromUrl = extractKeyFromUrl(input);

    if (fromUrl) key = `ntpl_dod/assets/${fromUrl}`;
  }
  if (typeof key === "string" && key.length > 0) {
    try {
      await s3.deleteObject({ Bucket: BUCKET, Key: key }).promise();
    } catch (_) {}
  }
}

module.exports = {
  // uploadFileToR2,
  uploadBufferToR2, 
  getSignedUrl,
  getNonCachedSignedUrl,
  deleteObjectByUrlOrKey,
  normalizeToR2Key,
  extractKeyFromUrl,
};
