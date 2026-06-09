module.exports = {
    async up(db, client) {
        const collections = ["products", "orders", "productcategories"];

        for (const name of collections) {
            const exists = await db.listCollections({ name }).hasNext();
            if (!exists) {
                await db.createCollection(name);
            }
        }

        const productIndexes = await db.collection("products").indexes();
        if (!productIndexes.some(idx => idx.name === "sku_1")) {
            await db.collection("products").createIndex({ sku: 1 }, { unique: true, sparse: true });
        }

        const categoryIndexes = await db.collection("productcategories").indexes();
        if (!categoryIndexes.some(idx => idx.name === "name_1")) {
            await db.collection("productcategories").createIndex({ name: 1 }, { unique: true, sparse: true });
        }
    },

    async down(db, client) {
        // Rollback only created collections if they exist and are empty.
        const collections = ["products", "orders", "productcategories"];

        for (const name of collections) {
            const exists = await db.listCollections({ name }).hasNext();
            if (exists) {
                const count = await db.collection(name).countDocuments();
                if (count === 0) {
                    await db.collection(name).drop();
                }
            }
        }
    }
};
