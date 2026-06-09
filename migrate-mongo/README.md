Here is a complete, self-contained, and production-ready `README.md` guide that you can save directly into your `db-migrations` folder. It covers everything from system prerequisites to advanced data rollbacks.

---

```markdown
# MongoDB Database Migrations Guide

This directory houses the database migration suite for our platform. While our main API is built using Rust, migrations are handled via Node.js using `migrate-mongo`. This allows us to leverage battle-tested tooling to safely mutate NoSQL structures without risking data loss or downtime.

---

## 1. Prerequisites

Before running or creating migrations, ensure your local environment has the following components installed:

* **Node.js**: Version 18.x or higher (Recommended: Long Term Support / LTS version).
* **npm**: Node Package Manager (comes bundled with Node.js).
* **MongoDB**: A running MongoDB instance (local or Atlas cluster) with a valid connection URI specified in the root `.env` file.

To verify your Node.js installations, run:
```bash
node -v
npm -v

```

---

## 2. Directory Initialization

If you are setting this up from scratch in a new environment, follow these steps to install dependencies and configure the environment.

### Step A: Initialize Node.js & Install Modules

Run these commands inside the `db-migrations` directory:

```bash
# Initialize a clean package.json
npm init -y

# Install the migration utility and dotenv (to read the root .env file)
npm install migrate-mongo dotenv

```

### Step B: Initialize the Migration Setup

Generate the boilerplate configuration file and migrations directory structure:

```bash
npx migrate-mongo init

```

This command creates a `migrate-mongo-config.js` file and an empty `migrations/` folder.

---

## 3. Configuration Setup

To ensure Node.js reads the exact same database settings as your Rust backend, replace the contents of your newly generated `migrate-mongo-config.js` with the following code.

This configuration automatically looks **one folder upward** (`../.env`) to parse your centralized environment variables.

```javascript
// db-migrations/migrate-mongo-config.js
const path = require('path');
// Load environment variables from the root Rust workspace directory
require('dotenv').config({ path: path.resolve(__dirname, '../.env') });

const config = {
  mongodb: {
    // Reads MONGODB_URI from your shared .env file
    url: process.env.MONGODB_URI || "mongodb://localhost:27017",
    
    // Reads DATABASE_NAME from .env, falls back to "dev_db"
    databaseName: process.env.DATABASE_NAME || "dev_db",

    options: {
      useNewUrlParser: true,
      useUnifiedTopology: true,
    }
  },

  // The collection where MongoDB tracks executed migration records
  migrationsDir: "migrations",
  changelogCollectionName: "changelog",
  migrationFileExtension: ".js",
  useFileHash: false,
  moduleSystem: 'commonjs',
};

module.exports = config;

```

---

## 4. The Complete Migration Lifecycle & Workflow

Managing schema changes follows a strict 4-step pipeline: Check Status ➔ Create Script ➔ Implement Changes ➔ Execute Migration.

### Step 1: Check Current Database Status

Always check the state of your database before writing or applying any scripts:

```bash
npx migrate-mongo status

```

**Example Output (Empty Database):**

| Migration | Applied? |
| --- | --- |
| No migrations found |  |

### Step 2: Create a New Migration Script

Generate a timestamped migration script file. Let's create one to rename our product tracking fields:

```bash
npx migrate-mongo create rename_sku

```

**Output:**

```text
Created migrations/20260609210000-rename_sku.js

```

> **Note:** The prefix `20260609210000` is a precise chronological timestamp (`YYYYMMDDHHMMSS`). This naturally forces Git merges to keep migration paths ordered, eliminating sequential version number conflicts.

### Step 3: Implement the Migration Code

Open the newly created file inside `migrations/`. You will find an `up` function (for applying changes) and a `down` function (for reverting changes if a bug occurs).

Here is a detailed production example showing field renaming, type conversion, and setting default values:

```javascript
// migrations/20260609210000-rename_sku.js

module.exports = {
  /**
   * UP: Logic to transform data forward when deploying changes.
   */
  async up(db, client) {
    // 1. Rename a field safely across all existing documents
    await db.collection('products').updateMany(
      { quantity: { $exists: true } },
      { $rename: { quantity: 'inventory_count' } }
    );

    // 2. Add a default "status" field to any legacy items missing it
    await db.collection('products').updateMany(
      { status: { $exists: false } },
      { $set: { status: 'active', is_featured: false } }
    );
  },

  /**
   * DOWN: Logic to completely reverse the 'up' function modifications.
   */
  async down(db, client) {
    // 1. Reverse the field renaming
    await db.collection('products').updateMany(
      { inventory_count: { $exists: true } },
      { $rename: { inventory_count: 'quantity' } }
    );

    // 2. Clean up and remove the injected default fields
    await db.collection('products').updateMany(
      {},
      { $unset: { status: "", is_featured: "" } }
    );
  }
};

```

### Step 4: Execute Pending Migrations (`UP`)

To run all scripts that are currently local but have not yet been executed in your target database, run:

```bash
npx migrate-mongo up

```

**Output:**

```text
MIGRATE-MONGO >> migrated 20260609210000-rename_sku.js

```

If you re-verify the status via `npx migrate-mongo status`, you will see your script marked with its precise execution timestamp:

| Migration | Applied? |
| --- | --- |
| 20260609210000-rename_sku.js | **2026-06-09 21:05:14** |

---

## 5. Reverting and Rolling Back Data (`DOWN`)

If your Rust application starts throwing validation errors or crashes because of a database schema change, you can instantly roll back the single most recently executed migration script.

Run the down command:

```bash
npx migrate-mongo down

```

**Output:**

```text
MIGRATE-MONGO >> rolled back 20260609210000-rename_sku.js

```

Running `npx migrate-mongo status` will confirm that the database has successfully rolled back and the script is marked as **PENDING** once again:

| Migration | Applied? |
| --- | --- |
| 20260609210000-rename_sku.js | **PENDING** |

---

## 6. Golden Rules for Rust Developers

1. **Atomic Commits**: Always commit your Node.js migration file and your modified Rust DTO structs (`ProductDto`, `UpdateProductDto`) together in the **same Git pull request**.
2. **Never Edit an Applied Migration**: If a script has already run in production, *never edit its code file*. Instead, generate a brand new migration script with `npx migrate-mongo create` to apply secondary corrections.
3. **Keep Default Validations Aligned**: Ensure that any default values injected by your Node.js migration script (`$set: { status: 'active' }`) perfectly satisfy the `garde` validation criteria defined in your Rust models.

```

```