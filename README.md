# Goupiblog
*a no BS, no JS, no nonsense blog-like static website generator and provider.*

I've always found most blogging tools to be a bloated mess. Goupiblog aims to
fix this.

## What is this?

Goupiblog, despite its name, is whatever you make it out to be. It's a simple
template generator. A Goupiblog post is made out of three components:
- The page itself, written as a markdown file
- The static resources, like images and whatnot
- The common prelude, that will be the pages' template.

Goupiblog also needs a few additional pages:
- The home page
- ... That's all!

The goals for this project are the following:
- Provide a no nonsense solution to templated page generation. KISS!
- Work with strictly no site-wide JavaScript
- Understand that the internet should be made of web pages, not applications!
- Feature no emojis in the Readme or the docs. Though this repo
probably contains blazingly fast security vulnerabilities!

## How do I use Goupiblog?

Simple! For now, Goupiblog only features page generation. Here's how to do it:

``goupiblog $SOURCES $OUTPUT``

And that's it! Your pages are now ready to go. 

## How are directories structured?

Goupiblog sources contain the following:
- A site.toml file, defining a few site-wide variables
- A resources directory, containing site-wide resources
- The prelude.html file
- A posts directory, containing one directory per post.

A post's directory contains the following:
- ``index.md``, the page to be added to the template
- ``index.html``, if you don't feel like using MD
- A resources directory, for page-specific resources.
- The ``post.toml``, specifying post-specific variables

The target directory contains the generated website, as well as 
``last_updated.toml``, a file containing the timestamp for each page's
last update, to allow rebuilding already built pages.

## The workflow

In order to create your own Goupiblog, you need to:

- Set up your directory structure
- Create your prelude (Or use the one provided!)
- Set up site-wide options
- Create your first post
- Host the post on a simple file server

We'll get into each of the steps below.

#### The prelude

Your prelude is an HTML file that contains the template for your site. 
It may feature special tags that will be replaced during transpilation. 
In fact, it probably should, otherwise, you'll just serve the template!
Here is a list of tags:

| Tag                   | Replacement                           |
|-----------------------|---------------------------------------|
| ``<GoupiContent/>``   | The transpiled page content           |
| ``<GoupiSite/>``      | The name of the site                  |
| ``<GoupiTitle/>``     | The title of the post                 |
| ``<GoupiDesc/>``      | The short description of the post.    |
| ``<GoupiAuthor/>``    | The author of the post                |
| ``<GoupiDate/>``      | The upload or update date of the post |

#### The options

Here is a list of site-wide options:

```toml
[site]
url = "https://yourwebsite.tld"
name = "Your fancy schamcy website name here"
# Note: This variable is never read. If you use Goupiblog, it's GPL :)
license = "Whatever"
description = "Hi, welcome to my website!'; DROP TABLE USERS '"
use_post_title_as_page_title = true
```

As for the post-specific options:

```toml
[post]
title = "My rant about R4L that nobody asked for"
description = "Today I argue why we should start JS4L instead."
author = "Me, obviously."
```

#### The post

Your post will be a simple directory within the sources directory.
This directory's name will be used as the internal name of your post. The resources directory is also copied over one-to-one. For example:

```plaintext
sources/my-post-title/ ==> target/my-post-title/
 + index.md                + index.html
 + res/                    + res/
 |  + myimage.png             + myimage.png
 + post.toml               
```

## Useless info and random trivia. You can stop reading here.

Here's some tidbits about Goupiblog that I didn't know where to put.

- It's not written in Rust (putting this here for the SEO :] ) for any
particular reason. I just felt like it since I've written major projects in 
pretty much every other language. This is shameless resume padding.
- I might add some more features in the future, but my main goal remains the
same, KISS. This is because trying to set up some self-hosted services made
me go nearly mad from how awful the docs were and how needlessly complicated
the code was to understand at times.
- At first I wanted to write this application in C, but I didn't feel like
dealing with libraries in that awful, awful ecosystem. Doesn't help that I'd
be missing all of the modern niceness of Rust without dirty implementations.
I'll make a tagged union pattern matcher in pure ANSI C one day. Today is not
the day.
- While I usually have
a fairly anti-library stance for many reasons (microlibrary culture genuinely
set us back decades worth of human progress, this isn't even a joke), I felt
like implementing an entire Markdown parser was perhaps a bit out of the
project's scope.
- I believe in the idea of finished software. One day, Goupiblog will be able 
to do what I want it to do without bugs. When that day comes, I'll archive the
repository. The code is done. Finished software does not need maintenance.
1.0.0 will be the final version.
