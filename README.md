# Kweepeer: Interactive Query Expansion Service

[![Project Status: WIP – Initial development is in progress, but there has not yet been a stable, usable release suitable for the public.](https://www.repostatus.org/badges/latest/wip.svg)](https://www.repostatus.org/#wip)
[![Crate](https://img.shields.io/crates/v/kweepeer.svg)](https://crates.io/crates/kweepeer)
[![Docs](https://docs.rs/kweepeer/badge.svg)](https://docs.rs/kweepeer/)
[![GitHub release](https://img.shields.io/github/release/knaw-huc/kweepeer.svg)](https://github.com/knaw-huc/kweepeer/releases/)

## Introduction

The Globalise project requests an interactive query expansion webservice which expands terms in search queries with available synonyms and other suggestions. These expansions are returned to the caller, for display in a user interface, and the caller has control over which suggestions to accept or discard, offering a high degree of control over the final query. Please see [the original plan](PLAN.md) for details and initial discussion.

This repository holds the backend service and underlying library that provides query expansion, it is called *Kweepeer* (pronounceas /ˈkʋe.pɪːr/ or  /ˈkwe.peːr/) and named after a fruit known as Quince in English.


## Installation

### From source

Production environments:

```
$ cargo install kweepeer
```

Development environments:

```
$ git clone git@github.com:knaw-huc/kweepeer.git
$ cd kweepeer
$ cargo install --path .
```

Development versions may require a development version of
[analiticcl](https://github.com/proycon/analiticcl) as well, clone it alongside kweepeer and add a
`kweepeer/.cargo/config.toml` with:

```toml
#[dependencies.analiticcl]
paths = ["../analiticcl"]
```

### Usage

To use the webservice, run `kweepeer` and point it to a kweepeer configuration file.
For the command-line interface, run `kweepeercli` and point it to a kweepeer configuration file.
To start using the Rust library, run `cargo add kweeper` within your Rust project.

See [the kweepeer(1) man page](docs/kweepeer.1.scd) for further details.

### Configuration

See [the kweepeer(5) configuration man page](docs/kweepeer.5.scd).

## Architecture

This schema presents an architecture with some proposed expansion modules. The modules
will be written in Rust, building on a common API, and compiled into the query expansion service.
Module details may need to be worked out further.

```mermaid
flowchart TD
    app[/"Web Application (e.g. TextAnnoViz)"/]
    frontend["Query Expansion UI Component (frontend)"]
    backend[/"Query Expansion Webservice (backend)"/]
    search[/"Search engine (any software)"/]
    searchdb[("Search Index")]
    wrapper[/"Search wrapper service"/]

    app -- "initial search query" --> frontend

    frontend -- "search query (HTTP POST)" --> wrapper

    backend -- "expanded search query" --> wrapper
    search -- "search results" --> wrapper
    search --- searchdb

    wrapper -- "search query" --> backend

    wrapper -- "expanded search query (HTTP POST)" --> search
    wrapper -- "search results" --> frontend
    frontend -- "search results and expansions" --> app

    parser["Query parser"]
    subgraph modules 
        lookup["Expansion Lookup Module (in-memory hashmap)"]
        lexsimfst["Lexical Similarity Module 1 (FST, in-memory)"]
        lexsimanaliticcl["Lexical Similarity Module 2 (Analiticcl, in-memory)"]
        semsim["Semantic Similarity Module"]
        autocomplete["Autocompletion Module"]
        translator["Translation Module"]
        expansionmap[("Expansion Map")]
        fst[("Finite State Transducer")]
        lexicon[("(Weighted) Lexicon")]
        lm[("Language Model (Transformer)")]
        tm[("Translation Model (Transformer)")]
    end

    parser["Query Parser/Lexer"]
    compositor["Query Compositor"]
    expander["Term Expander"]
    parser -- "search terms" --> expander

    backend --> parser
    compositor -- "expanded search query + terms + template" --> backend

    expander <-- "search term" --> lexsimfst
    expander <--> lookup
    expander <--> lexsimanaliticcl
    expander <--> semsim
    expander <--> autocomplete
    expander <--> translator
    expander -- "expanded search terms + template" --> compositor

    lookup --- expansionmap
    lexsimfst --- fst
    fst --- lexicon
    lexsimanaliticcl --- lexicon
    lexsimanaliticcl --- expansionmap
    semsim --- lm
    autocomplete --- lm
    translator --- tm

    classDef external fill:#ccc,color:#111
    class app,search,searchdb external
```

### Modules

The following modules are implemented:

* *Category: Syntactic Similarity*
    * **Lookup Module** -- `lookup` -- A simple lookup module that loads a mapping of terms and expansions from file into memory, and does lookup against it at run-time.
    * **Finite State Transducer Module** -- `fst` -- Takes a lexicon as input and uses a Finite State Transducer to identify possible expansions from the lexicon within a given edit distance.
    * **Anagram-hashing Module** -- `analiticcl` - Takes a lexicon or variant list as input and uses anagram hashing and further techniques to identify similar terms. This also has various advanced options such as the ability to define confusable characters, and simple language modelling capabilities. It uses [analiticcl](https://github.com/proycon/analiticcl).
* *Category: Semantic Similarity*
    * **Semantic Similarity** -- `finalfusion` -- This implements semantic similarity using word embeddings and vector comparison metrics. It uses [finalfusion](https://github.com/finalfusion/finalfrontier).


