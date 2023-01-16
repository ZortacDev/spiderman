# spiderman: The weaving project manager

Spiderman manages your projects by associating each project with a set of _tags_ and dynamically generating 
set of _views_ based on these tags.

All projects live within a spiderman project root. All subcommands (except **init** which creates a new root) 
operate with respect to the _current_ project root. The project root is either the next upstream 
directory (from the current working directory) that qualifies as a project root, or the default project root, 
which can be set in `~/.config/spiderman/config.toml`.

A spiderman project root contains a `.spiderman` directory, which, in turn, contains the `raw` directory, 
holding all the projects managed by spiderman, and the `schema.toml` configuration file, describing how 
spiderman should build view trees based on tags.

The `schema.toml` file contains a list of schemas and a set of default tag values to use when a 
project does not have a tag, but that tag is used in a schema. Schemas consist of `/` separated components 
that are either plain strings, in which case they will be used in the view tree verbatim, or strings contained 
in curly braces (`{` and `}`) in which case the string is interpreted as the name of a tag and substituted 
with the appropriate tag value when building the view tree.

A project's tags are specified in its `spiderman.tags` file, which can be edited using the **tags** subcommand. 
Each line in this file consists of colon (`:`) separated values. The first of these is the name of the tag, while the 
later ones are values for that tag. A project may have multiple values for one tag.

## Example
`schema.toml`:

```toml
schemas = [
    "by-organization/{organization}/{type}",
    "by-type/{type}/{organization}"
]

[default_tag_values]
organization = "Personal"
type = "Random"
```

`spiderman.tags`:
```
organization:Uni
type:Software:Writing
```