# Niri screenshot tools
A little program that runs in the background and lets you use the Built-in niri screenshotter with annotators like satty or swappy.
It also supports uploading screenshots to a copyparty instance, although more services could be added in the future

## Config

Here's an example config:
```toml
[annotator]
command = 'satty --filename "%path%"'
enabled = true
auto = true

[uploader]
url = "https://example.com/imgs/%name%"
enabled = true
auto = true
```
### Annotator
* `command`
  * Shell command to run the annotator, with `%path%` being replaced by the filepath
* `enabled`
  * Whether the annotator will get triggered at all
* `auto`
  * If true, the annotator will automatically open with the screenshot you took. If false, it will send a notification that opens the annotator on click

### Uploader
(Note that currently, only instances of copyparty are supported. It just uses a PUT request though, so it may well work with other things)
* `url`
  * The url of the location to upload the screenshot to, with `%name%` being replaced by the base name of the file
* `enabled`
  * Same as Annotator
* `auto`
  * Same as Annotator
