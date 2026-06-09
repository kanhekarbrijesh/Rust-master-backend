// db-migrations/migrate-mongo-config.js
const path = require('path');

// 1. Core Fix: Step up one folder level to find and load the root .env file
require('dotenv').config({ path: path.resolve(__dirname, '../.env') });

// Clean up any literal wrapping quotes that might be parsed from the .env file
const rawUri = process.env.MONGO_URI || "mongodb://localhost:27017";
const mongoUri = rawUri.replace(/^["']|["']$/g, "");

const config = {
  mongodb: {
    // 2. Core Fix: Dynamically match your exact environment variable keys
    url: mongoUri,
    databaseName: process.env.MONGO_DB_NAME || "rusttut",

    options: {
      // connectTimeoutMS: 3600000, // increase connection timeout to 1 hour
      // socketTimeoutMS: 3600000, // increase socket timeout to 1 hour
    }
  },

  migrationsDir: "migrations",
  changelogCollectionName: "changelog",
  lockCollectionName: "changelog_lock",
  lockTtl: 0,
  migrationFileExtension: ".js",
  useFileHash: false,
  moduleSystem: 'commonjs',
};

module.exports = config;