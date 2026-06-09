1. Identity & Access Management
Pair: User ↔ Role

Relationship: Many-to-Many

MongoDB Approach: Usually modeled as an array of Role strings or ObjectIds embedded directly into the User document for fast authorization checks.

6. EdTech & Learning Management
Pair: Course ↔ Syllabus_Module

Relationship: One-to-Many

MongoDB Approach: Modules represent the structure of a course. They are highly structured and static, making them perfect candidates for direct embedding inside the Course document.

7. Media & Entertainment
Pair: Artist ↔ Album

Relationship: One-to-Many

MongoDB Approach: An album can be a standalone entity with its own metadata (release year, cover art), so it's usually stored in its own collection with an artist_id reference.

8. Fintech & Banking
Pair: Bank_Account ↔ Transaction

Relationship: One-to-Many

MongoDB Approach: Transactions grow infinitely over time. Embedding them would breach MongoDB’s 16MB document limit. They must be separate documents in a transactions collection referencing a bank_account_id.

9. HR & Workforce Management
Pair: Employee ↔ Timesheet

Relationship: One-to-Many

MongoDB Approach: Weekly or monthly timesheets are typically handled as separate documents referencing the employee, allowing HR admins to query timesheets across the entire company easily.

10. Content & Social Media
Pair: Blog_Post ↔ Comment

Relationship: One-to-Many

MongoDB Approach: Depending on scale. For low-traffic blogs, comments are embedded. For highly viral content, comments are offloaded to a separate collection to prevent document bloating.

11. Real Estate & PropTech
Pair: Property ↔ Amenity (e.g., Pool, Gym, Garage)

Relationship: Many-to-Many

MongoDB Approach: Amenities are usually a fixed master list, so storing an array of simple strings or IDs like ["pool", "gym"] inside the Property document is standard practice.

12. Fleet & Asset Management
Pair: Vehicle ↔ Maintenance_Log

Relationship: One-to-Many

MongoDB Approach: Maintenance events happen periodically. They can either be embedded as a bounded array for quick vehicle history retrieval or separated if they contain heavy invoice data.
 