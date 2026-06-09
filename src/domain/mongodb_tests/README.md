

2. E-Commerce Catalog
Pair: Product ↔ Category

Relationship: Many-to-Many

MongoDB Approach: Products typically embed an array of Category IDs or slugs to keep catalog browsing lightning-fast without joins.

3. Geography & Localization
Pair: Country ↔ State

Relationship: One-to-Many

MongoDB Approach: Because the number of states in a country is bounded and changes rarely, states are often embedded as an array of sub-documents inside the Country document.

4. Logistics & Supply Chain
Pair: Shipment ↔ Tracking_Event

Relationship: One-to-Many

MongoDB Approach: A shipment generates a historical log of status updates. These are safely embedded as an array of sub-documents since a single shipment rarely has more than a few dozen updates.

5. Healthcare & Digital Health
Pair: Patient ↔ Medical_Record

Relationship: One-to-One

MongoDB Approach: Due to strict privacy boundaries and document size limits (medical records can grow massive with history), the Medical_Record is usually a separate document referencing the Patient via an ObjectId.
