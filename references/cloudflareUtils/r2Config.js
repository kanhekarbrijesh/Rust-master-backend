const AWS = require('aws-sdk');
const dotenv = require('dotenv');
dotenv.config();

/**
 * Configure and initialize the AWS S3 client for Cloudflare R2
 */
const s3 = new AWS.S3({
  endpoint: process.env.R2_ENDPOINT,
  accessKeyId: process.env.R2_ACCESS_KEY,
  secretAccessKey: process.env.R2_SECRET_KEY,
  // region: "auto",
  signatureVersion: "v4",
  s3ForcePathStyle: true,
});

/**
 * Determine MIME type based on file extension
 * @param {string} file - Filename with extension
 * @returns {string} MIME type
 */
function getMime(file) {
  return file.endsWith(".m3u8")
    ? "application/vnd.apple.mpegurl"
    : file.endsWith(".ts")
    ? "video/MP2T"
    : file.endsWith(".jpg")
    ? "image/jpeg"
    : "application/octet-stream";
}

module.exports = {
  s3,
  getMime
};
