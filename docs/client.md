# Overview
In this toutrial your going to learn how to view served documents
## view
### viewing a document normally (no tui)
this viewing method spits out text and the file that their in with no good tui
```sh
nexadu view <document-id> <current|version>
```
### viewing with tui
you can view a document within a sepecfic version with the command:
```sh
nexadu view <document-id> <current|version> tui
```
(also the tui currently contains mark down support)
### viewing within a network
you can view your document with in a network with the following command:
```sh
nexadu view <document-id> [vesion] --remote <address:port>
```
or if you wany tui:
```sh
nexadu view <document-id> [vesion] tui --remote <address:port>
```
## spans&reference
### viewing spans
you can view a span within a document:
```sh
nexadu span <document-id> [version] <span-id> [all]
```
### listing spans
you can list spans with:
```sh
nexadu spans-list <document-id>
```
### viewing references
```sh
nexadu ref <document-id> <ref-id>
```
### listing references
you can list references with:
```sh
nexadu ref-list <document-id>
```
