+++
title = "drg for easy drogue cloud administration tasks"
extra.author = "jbtrystram"
description = "A guided tour of `drg` the command line client for drogue-cloud."
+++



A guided tour of `drg`, the command line client that aims to make your life easier when interracting with a `drogue-cloud` instance.

<!-- more -->

`drg` is the newest addition to the drogue familly. The idea behind it is to make interractions with the drogue-cloud APIs less cumbersome and more intuitive to use. 
Obviously it's written in rust, and try to have an intuitive use flow, insparied from the famous `kubectl` tool. 
We landed some nice new features recently, enough to warrant a release. Get yourself the lastest version, and hop on for a tour !


Note: If you used `drg` before version `0.4.0` then you MUST delete your configuration file, as its format was changed to support multiple environments. 
By default it should be `$HOME/.config/drg_config.json`. (Or whatever your `XDG_CONFIG_HOME` specifies.) It's now a yaml file with a different structure.

## Installing drg

If you have a rust toolchain available you can install `drg` from [crates.io](https://crates.io/crates/drg): 

     cargo install drg

It will download and build you the latest released version. 

If you don't have rust on your machine you can download a binary from the [Github release](https://github.com/drogue-iot/drg/releases) page.
At this time we have Linux and MacOS binaries available. 

Alternatively and if you are a mac user you can get drg through homebrew : 

    brew tap drogue-iot/drg
    brew install drg


Windows support is coming soon ! 

## Log in your drogue-cloud cluster

Now that you have a running `drg` you can log into your cluster : 

    drg login https://api.drogue-cloud-cluster.tld


You'll be guided to log in into your cluster using OAuth2, in your usual browser. 
At this point, `drg` will ask you to name this context. This allows you to have multiple instance of drogue-cloud configured and switch back and forth. More on that later ! 


## Create and manage applications

As you may already know, devices communicate with drogue-cloud in a scoped environnemeent, called an Application. You can read more about the drogue cloud concepts in the [documentation](https://book.drogue.io/drogue-cloud/dev/concepts.html).

Let's start by creating an application for the things in my house in our drogue-cloud instance : 

    ~ drg create app house
    App house created.


That is neat but it is definitely missing some context. We can also create an entry with some more context in the `spec` section :
    
    ~ drg create app french-house --spec '{"location":"france"}'
    App french-house created.


You can also read data from a file: `drg create app house -f /path/to/json/file`. Please note that in that case the json file must describe the whole object. Here is an example : 
```json
{
  "metadata": {
    "name": "french-house"
  },
  "spec": { 
    "location": "france"
  }
}
```

But let's say I moved to a new country as I need to improve my german. Let's fix that ! `drg edit` will spawns you a neat editor : 
[![asciicast](https://asciinema.org/a/LiPIT2S22pP3MCcsZS9SNlZaF.svg)](https://asciinema.org/a/LiPIT2S22pP3MCcsZS9SNlZaF)



Applications can also be updated reading files : `drg update app -f /path/to/app/json/file`. 
Make sure you retrieve the full metadata object before doing an update (`drg get app house` will help you do that) because 
you need to provides some other data for an update, such as the unique id of the app an it's resource version !


And of course, if you sell your house you can always remove it from the system : 
     
     drg delete house french-house
 
 
## Create and manage devices

So now that we have a good idea of how to manage our house scope let's add devices : 

    drg create device fridge --app=house

The behaviour is the same as for the applications so you can create from a file, update and edit and so on..

    drg create device coffee-machine --app=house -f /path/to/device/json
    drg update device coffee-machine --app=house


I am starting to get annoyed by the `--app` argument ? It's uneedlessly repetitive. Let's pull it from the environment: 

    export DRG_APP=house
    drg create device fridge

The app setting is now pulled from the env variable. Less keystrokes ! Yay ! 
and it can also be pulled from the context, that leads nicely into the configuration section, see below.

    
## Contexts and configuration file

As I said in the beginning `drg` can keep track of multiple clusters if the need arises. Let's explain how that works. 
The configuration file contains multiple contexts. A context represents a cluster API endpoint, a default application (optional) and an authentication token (0Auth2).

When we logged into a cluster at the begining we saved a context with a name. Let's say we named it `my-cluster`, we can set a default app for it : 
     
     drg context set-default-app house --context=my-cluster


Note: here the `--context` argument is optional as a default context can be pulled from the environment variable `DRG_CONTEXT`. 
Also, when you log into a cluster without previous info in the config file (.i.e. running `drg login`for the first time), the new context will be set as active by default. 

But `my-cluster` is not a really great name, as we just moved it to an Azure cluster, let's make it clearer : 
    
    drg context rename my-cluster azure.
    
You can list, delete, set another context as active and show the config file. 

# What's next ?

We have a nice long list of things that we want to add in `drg` !
Here are the high priority items : 
 * list devices and apps
 * Support JSON PATCH to update resources
 * Sending commands to devices using the [command and control feature](https://blog.drogue.io/command-your-devices/) of drogue cloud.
 * Tap into a stream of telemetry and see the stream on your console.

You can go and look [in the issue section of the repo](https://github.com/drogue-iot/drg/issues) for more details and submit your awesone ideas there.
