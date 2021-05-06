+++
title = "The Book: documentation and workshops"
extra.author = "ctron"
description = "foo"
+++

Writing code is easy, writing documentation is hard. True, some people might disagree with that. We wanted to make it
easier for everyone to understand what we already have, and how it works. As we have multiple major topics
(embedded and cloud), we need a modular way of structuring our documentation. Here is our approach. 

<!-- more -->

# The Book

Instead of having multiple documentation sites, and multiple locations for the different released version, we wanted
to have a single point for you to go to when it comes to documentation of Drogue IoT. A single book.

From a technical perspective [Antora](https://antora.org/) provides all that we need. Multi-version support,
multiple content sources, theming, Git integration (edit this page), and the support for AsciiDoc.

## Structure

The two main components of Drogue IoT are "Drogue Device" and "Drogue Cloud". Each of the components needs their own
structure and has their own release cycle. Then again, there are some overarching topics, and it would be nice to
reference from one component to content in the other.  So we have:

* [An introduction section](https://book.drogue.io/drogue-book/), as part of the main "book" repository
* A main section on [Drogue Device](https://book.drogue.io/drogue-device/dev/index.html)
* A main section on [Drogue Cloud](https://book.drogue.io/drogue-cloud/dev/index.html)
* And a [workshops](https://book.drogue.io/drogue-workshops/index.html) section, featuring more hands-on/walk-through
  style content, which combines all the aspects of Drogue IoT.

## Workshops

While we currently have only on workshop, LoRaWAN end-to-end, I still want to highlight this section in this blog post.

Writing good documentation is hard. What is even harder, is to create useful content for the end user, you! Content
which enables you to use our technical bits and pieces to solve some problems. More user or use-case focused content,
bringing it all together.

So we would encourage you to take a look. But, don't stop there. If you have ideas on your own workshops or tutorials,
reach out to us! Or, even better, contribute a workshop yourself!

## Also see

* [The Book](https://book.drogue.io)
  * [drogue-iot/book.drogue.io](https://github.com/drogue-iot/book.drogue.io)
  * [drogue-iot/drogue-device - /docs](https://github.com/drogue-iot/drogue-device/tree/main/docs)
  * [drogue-iot/drogue-cloud - /docs](https://github.com/drogue-iot/drogue-cloud/tree/main/docs)
  * [drogue-iot/drogue-workshops](https://github.com/drogue-iot/drogue-workshops/)
