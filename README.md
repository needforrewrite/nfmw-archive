# nfmw-archive
Archive server for NFM World. Stores the following:

- Cars
- Stages, and stage music
- Custom stage pieces
- Custom car wheels
- Associated metadata files

The actual filenames of the item and associated .meta file are randomised, but paired. Each item type is sorted into a folder. For stages, the music shares the same filename with the appropriate filetype.

Author information is stored but there is no form of authentication. For now, the uploading of items to the production instance is handed by the developers of NFM World and anyone else deemed trustworthy enough.

No items should be uploaded without the item creator's consent, except if the item is originally Super Public in NFMM, or has previously been released for use in the public domain.


## Usage

config.toml (based from config.template.toml) stores the *absolute* path to the directory where items will be stored. 
This directory can be directly exposed within a webserver in order to enable downloading of the files.
The server checks if a `nfmw-archive` directory exists at the path. If it does not, it will make one and use that.

config.toml also stores the server port and authentication key. To upload, delete or modify files, send a PUT, DELETE, or PATCH HTTP request as appropriate.

### Create item

PUT /item

Request body must be the item itself.
Query parameters are all for metadata. Metadata query parameter names described below.

### Modify item

PATCH /item/:id

The ID is the appropriate filename. Query parameters describe the metadata properties to modify. To modify the item itself, delete it (below) and reupload it.

### Delete item

DELETE /item/:id

The ID is the appropriate filename.

### Search for items

SEARCH /item

Query parameters for metadata can be used as a match. At least one must be defined. Presently, only author, name, and type are valid filters.

Result count is limited to 10. Pagination is suppored by using ?page=x where x is the page number. The number of pages is reported in the response header X-Search-Pages.

This request is unauthenticated. Any given IP can only search once per 2 seconds.

## Metadata

Each stored item also has an associated .meta file holding metadata, including:

- Item name (`name`)
- Author name or alias (`author`)
- Date uploaded (`uploaded`, returned as ISO-8601 timestamp)
- Date created (`created`, if known. Same as `uploaded` if not provided. Returned as ISO-8601 timestamp.)
- Type of file (`type`)
- Comment (`comment`, can be any string up to 250 characters long)

Some types of file can also have additional metadata stored with them, for example cars can have an entry saying the model they are based off (if not indicated by the car's in-game name).
All of these properties are optional, unless stated otherwise. If ommitted, they are not stored in the metadata file.

For cars:

- Model (`model`)
- Recharged (`recharged`, optional, but defaults to 0, must be stored as int 0 (false) or 1 (true))

For wheels, stages, and custom stage pieces, there is at present no specific metadata.
