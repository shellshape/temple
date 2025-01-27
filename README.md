# temple

A **very** simple static web page generator to build simple static web pages without the burden of code duplication.

This tool is primarily built for [my personal website](https://github.com/zekroTJA/new.zekro.de).

## Usage

Temple basically used 3 components for building your website.
- `templates`: HTML files which can be used as foundation or components for your webpage.
- `pages`: HTML files which can *use* or *extend* from `templates`.
- `public`: Public source or media files like stylesheets, scripts, images, ...

The required project layout in the *source* directory looks as following.
```
src/
    pages/
        0_index.html
        1_projects.html
        2_contact.html
        imprint.html
        ...
    public/
        stylesheet.css
        somescript.js
        favicon.ico
        site.webmanifest
        ...
    templates/
        base.html
        ...
```

A page or template is a simple HTML file which can also have functions and a config.

Lets assume you have a template file `src/templates/base.html` which has the following content.
```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <title>zekro.de | {{pagename}}</title>
  </head>
  <body>
    <nav class="header">
      {{navitems}}
    </nav>
    <main>{{pagecontent}}</main>
    <footer>some footer</footer>
  </body>
</html>
```

And you have a page `src/pages/0_index.html` with the following content.
```html
+++
title = "hey"
path = "/"
output = "index.html"
+++

{{ extends base }}

<h1>Hey!</h1>
```

And maybe another page page `src/pages/1_projects.html` with the following content.
```html
{{ extends base }}

<h1>Projects!</h1>
```

When building the page, the following output will be generated.
```
dist/
    index.html
    projects/
        index.html
    public/
        stylesheet.css
        somescript.js
        favicon.ico
        site.webmanifest
        ...
```

And the contents of the file `dist/index.html` will looks as following.
```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <title>zekro.de | hey</title>
  </head>
  <body>
    <nav class="header">
      <a href="/" class="active">hey</a>
      <a href="/projects">projects</a>
    </nav>
    <main><h1>Hey!</h1></main>
    <footer>some footer</footer>
  </body>
</html>
```

And the contents of the file `dist/projects/index.html` will looks as following.
```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <title>zekro.de | projects</title>
  </head>
  <body>
    <nav class="header">
      <a href="/">hey</a>
      <a href="/projects" class="active">projects</a>
    </nav>
    <main><h1>Projects!</h1></main>
    <footer>some footer</footer>
  </body>
</html>
```

### Functions

#### `{{ extends <template_name> }}`

Replaces `{{ pagename }}` in the template given via the `template_name` with the contents of the page. This is useful to build a scaffolding for your web page to extend your pages content into.

#### `{{ use <template_name> }}`

Is replaced by the content in the template with the passed `template_name`. This is useful for components which are used in multiple pages of your site.

#### `{{ pagename }}`

Will be replaced with the name of the current page.

#### `{{ navitems }}`

Will be replaced with an anchor list with links and the names of all pages. When `navignore` is set to `true` in the page config, the page will not appear in the list.

> [!TIP]  
> When a page name has an underscore in the name, everything before the first underscore and itself will be removed from the name. This way you can sort the pages so that the `{{navitems}}` function always ensures the same order. 

#### `{{ currentdate <format?> }}`

Will be replaced with the current date, formatted with the given `format` string. When no format string is given, the default format of `%Y-%m-%d %H:%M:%S` will be used. [Here](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) you can find the full specification for the date format.

#### `{{ exec <command> <args...> }}`

Executes a `command` with the given `args` and will be replaced with it's outputs.

> [!WARNING]  
> Please be cautious with this feature, especially in CI/CD pipelines, as it wil lexecute everything passed without any filtering!

### Page Config

As you see above, you can configure some stuff of your pages by putting it in a block at the top level of the page beginning with `+++` and ending with `+++`. The contents of the block are in [TOML](https://toml.io/en/) format.

```toml
# The page title.
# When not set, the name of the file will be used.
title = "hey"

# The navigation path of the page. 
# When not set, the path will be "/<title>".
path = "/somethingelse"

# Alternative output directory in the destination directory.
# When not set, the output path will be "<pagename>/index.html".
output = "index.html"

# When set to true, the page will not be listed in the 'navitems' function.
navignore = true
```

### Real World Example

If you need a real world example, my personal web page is built with this tool!

- [Page](https://zekro.de)
- [Repository](https://github.com/zekrotja/new.zekro.de)

## Install

You can either download the latest release builds form the [Releases page](https://github.com/shellshape/temple/releases) or you can install it using cargo install.
```
cargo install --git https://github.com/shellshape/temple
```