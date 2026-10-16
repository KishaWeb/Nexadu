# Overview
## How to serve your documents
this is a breif tutorial on how to serve your first documents on Nexadu
## Initializing the documents
Initializing the documents is pretty easy the command for it is:
```console
nexadu init <your-document>
```
and your document is now in ```~/.Nexadu``` (also containing versions, refrences, spans and metadata)
## Version control
### Making a new version of your document
You can make a new version of your document with:
```sh
nexadu version <your-document-id>
```
### Restoring
you can restore a version/go back to a version with:
```sh
nexadu restore <your-document-id> <the-version-you-want-to-go-back-to>
```
### Check status
you can check the version status and changes of your document with:
```sh
nexadu status <your-document-id>
```
### History
you can check the version history of your document with:
```sh
nexadu history <your-document-id>
```
## Spans
spans are a specific chunk of text that also display refrences.
### Creating a span
you can creat a span with
```sh
nexadu span-create <your-document-id> <version|current> <file> <start> <end>
```
### Displaying a span
you can see a span with
```sh
nexadu span <your-document-id> <version|current> <span-id>
```
### List of spans
you can see a list of spans in a document with:
```sh
nexadu spans-list <your-document-id>
```
### Removing a span
you can remove a span with: 
```sh
nexadu span-remove <your-document-id> <span-id>
```
## refrences
refrences are links in between files and documents that go back to each other.
### creating a refrences
you can make a refrence with
```sh
nexadu ref-create <from_document> <from_version> <from_span> <to_document> <to_version> <to_span>
```
### display a ref
you can see a refrences with:
```sh
nexadu ref <your-document-id> <reference>
```
### Listing references
you can see a list of references in a document with:
```sh
nexadu ref-list <your-document-id>
```
### Removing a reference
you can remove a reference with:
```sh
nexadu ref-remove <refrence-id>
```
## networking
### serve a document in your LAN
you can serve a document in your LAN with:
```sh
nexadu serve <address>:<port>
```
