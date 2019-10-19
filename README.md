# kyusa
kyusa is a ShareX server written in rust.


## Endpoints
- /{folder}/{file}
	- kyusa stores all media as a random file name in a random folder.
	- Non media files will maintain their filename on upload.
-  /d/{folder}/{file}?={key}
	- This is the endpoint to delete the file.
	- Full link is returned on successful upload and is stored in the sled db.
- /u
	- This is where you will send the multipart uploads to.
	- The file form name is "file"
- /stats
	- This shows the current version along with git commit

## Features

 - Self contained, using sled as a database.
	 - This keeps everything running super fast without any need for having to configure a database provider.
- Fast, written in rust and actix-web this keeps thing running as quick as possible.
- Delete keys, whenever you upload a imagine you are returned a link to delete the image from the server.
- CloudFlare cache support, if your website is behind the CloudFlare cdn even after you delete the image it will still exist on the CDN.
	- Whenever you delete an image it kyusa will poll the CloudFlare api and purge that image.

## Planned Features

- API Authentication
	- Currently server is setup as public, however adding this will allow you to add users via an api endpoint.
- Better stats
	- Add total size of hosted images
	- Add total bandwidth served
 
