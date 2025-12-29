# nfmw-archive
Archive server for NFM World. Stores the following:

- Cars
- Stages (and their music, stored separately and not treated as an item)
- Stage pieces
- Custom car wheels

The server has two components - the item filesystem holding all files, and a PostgreSQL-based index for searching.

## High Level Design

The main source of truth is the file system. Each item, in its file, must hold all information relating to that item.

In order to facilitate searching, some information is aggregated from the files into index tables. For example, each item may have one or multiple authors - we can create a table with two columns - one for the item ID (filename) and one for an author ID. A item ID can appear multiple times with different authors, and a single author can be credited for multiple items. When a new item is added, that item is then added to the index table. The index table can also be manually regenerated as and when needed. This is split into item types - so, for example, there would be an index linking a car ID to authors, and a different table linking stage IDs to authors.

Authors are anyone who is credited in the author() tag in the item. When a new author is found, they are added to an author index table linking their name to an author ID, which is then used in the other indexing tables linking that author ID to item IDs.

There is one table which is not an index, and that is the table tracking who uploaded each item. This is a table linking a user ID (separate from author ID) to an item ID, and again has one table for each type of item. This table controls how items are edited and deleted by making sure only the uploader (or server admins) can do so.


### Filesystem

The filesystem divides the items by their type. For example, all cars live in cars/. Within cars/, cars are organised by their collection. By default, all uploaded items go into the WorldUser collection. Only admins can move items to other collections. 

Collections may be added or removed over time, and there is no static list of available collections at any one time, but the available collections must be synced between NFMW and the archive server - the archive server will accept anything as a collection, but the game is quite rigid on its collection definitions and will not be able to query anything in collections it does not recognise.

### Account Management

This server also acts as the account management system. Each stored user has a username, a hash of their password and the salt of that hash. When the user tries to log in, the provided password is hashed and salted. If it matches the stored hash, the attempt is valid and the user is provided with an authentication token.

When the client uses this token, it must first generate a random 64-byte long salt and then hash the token using that salt. Then, the client provides the username, hashed and salted token, and the salt. The server salts and hashes the stored token for that username, and if the hashes match, the token is valid. The salt can only be used once. This prevents any middleman from storing the token and attempting to use it as authentication to other services.

In the context of the archive server, every attempt to download, upload or change a stored file counts as use of a salt and as such it should be regenerated after each.