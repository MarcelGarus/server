topics: Chest, Dart, code

# Chest

## A new database

Databases are a fundamental part of most modern applications.
Studying [Hive](https://hivedb.dev) interested me in database design, so I decided to implement a new database – called *Chest*.  
I created it as a research project, but I'd also be happy to produce something worthwhile.

...

These are the goals:

- Implement a *pure-Dart* database from scratch.
- Support storing *significant amounts of data* that don't fit into RAM.
- Offer a *NoSQL*-API similar to Firestore.
- Support *queries* efficiently.
- Have excellent *developer experience*.

These are non-goals:

- Don't optimize for server use-cases, but focus on *clients first*.
- *Don't offer native sync capabilities* with cloud services or peer-to-peer.
- *Don't focus on performance* over everything else.

I want to emphasize the last non-goal especially.
That's not to say I don't care about speed at all – I'll try to keep the time complexity to a minimum and create an architecture that's not inherently slow. But I'll postpone low-level performance optimizations to a (much) later date.

Looking at the goals, this will – unlike Hive – not be an in-memory database but a resource-efficient, full-fledged database that supports vast amounts of data.
In this series, I'll document my architecture bottom-up and build lots of small layers of abstraction that each bring us closer to the vision of a complete database.
