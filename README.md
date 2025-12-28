# nfmw-archive
Archive server for NFM World. Stores the following:

- Cars
- Stages (and their music, stored separately and not treated as an item)
- Stage pieces
- Custom car wheels

The server has two components - the item filesystem holding all files, and a PostgreSQL-based index for searching.

## Design

The main source of truth is the file system. Each item, in its file, must hold all information relating to that item.

In order to facilitate searching, some information is aggregated from the files into index tables. For example, each item may have one or multiple authors - we can create a table with two columns - one for the item ID (filename) and one for an author ID. A item ID can appear multiple times with different authors, and a single author can be credited for multiple items. When a new item is added, that item is then added to the index table. The index table can also me manually regenerated as and when needed. This is split into item types - so, for example, there would be an index linking a car ID to authors, and a different table linking stage IDs to authors.

Authors are anyone who is credited in the author() tag in the item. When a new author is found, they are added to an author index table linking their name to an author ID, which is then used in the other indexing tables linking that author ID to item IDs.

There is one table which is not an index, and that is the table tracking who uploaded each item. This is a table linking a user ID (separate from author ID) to an item ID, and again has one table for each type of item. This table controls how items are edited and deleted by making sure only the uploader (or server admins) can do so.