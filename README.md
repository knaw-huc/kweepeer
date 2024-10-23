# Interactive Query Expansion Service

## Introduction

The Globalise project requests an interactive query expansion webservice which
expands terms in search queries with available synonyms and other suggestions.
These expansions are returned to the caller, for display in a user interface,
and the caller has control over which suggestions to accept or discard,
offering a high degree of control over the final query.

The system should be set up in such a way that we can can experiment with different
query expansion mechanisms.

Note that the service proposed here concerns only the backend system in this stage.

## Architecture

I see two architectural options:

1. The expansion service may takes the form of a query rewrite service and sits as a mediator between the 
   frontend and the actual search index:
	* It receives a query (e.g. using Lucene query syntax), 
    * The query is parsed (e.g. using ANTLR) and terms identifies
    * Expansions for terms are computed
    * A new query is formulated with disjunctions of terms
    * The query is sent to the search engine
    * The search response is propagated back to the called, after marking all applied expansions (e.g. in the HTTP response header).
   It is then an easily pluggable solution. This is how the idea was proposed.
   It is up to the caller to formulate the initial query.
2. The expansion service is decoupled from the actual search engine.
    * It receives one of more terms
    * Expansions for the search terms are computed
    * Expansions are returned
   In this set-up the role of the expansion service is more minimal and focusses only on term expansion.
   It is up to the caller to formulate the final query.

Arguments for/against option 1:

* Less burden on the frontend, it communicates only with the expansion service (but still it has to implement an editor for all the suggestions)
* The tighter coupling may allow search indexes from the actual search engine to be reused by the expansion service.
* This fits the nomer 'query expansion service' better

Arguments for/against of option 2:

* The expansion service is more minimal/simpler, much easier to implement/maintain
  as it decouples from any query language and query engine.
* Easy to reuse in other wider contexts.
* There is a higher burden on the caller to formulate the query, but:
* The caller also has more control over the query
* Separate search indices may be needed for certain expansion solutions, though even in this scenario we do retain the option to  reuse existing indices.

**Question 1:** *What do we prefer? Are there more arguments that play a role?*

> (Maarten): I currently have a preference for option 2 as it is more minimalistic and decoupled

## Technologies

The service will be implemented in Rust and focus on performance.

* For Architecture Option 1, query parsing can be done through binding with the [ANTLR C++ runtime](https://github.com/antlr/antlr4/blob/master/doc/cpp-target.md) (this has to be investigated).
* For query matching against lexicons (e.g. INT historical lexicon), options are:
    * [FST library](https://github.com/BurntSushi/fst).
    * [Analiticcl](https://github.com/proycon/analiticcl)
    * [Tantivity](https://github.com/quickwit-oss/tantivy) - A full text search engine library, makes most sense in architecture option 1
    * More semantic expansions (Sparse Vector Search) can also be considered.

The idea is that multiple expansion mechanism can be explored. The Rust API
should offers the right level of abstraction and flexibility so new modules can
be plugged in.

**Question 2:** *Depending on the expansion mechanism and the size of its model, in-memory models may be sufficient. This will benefit performance. What do you think?*

> (Maarten): I have a preference for (efficient) memory-based models when possible. I also prefer minimising dependencies, especially infrastructural once such as external services (like databases).

## Further Questions

**Question 3:** *Though this is a backend-project, it might be good to take frontend development and wishes into account at an early stage. How is it going to be integrated for instance in TextAnnoViz?*

> (Maarten): I think a dedicated (reactjs?) component may be envisioned to communicate with the proposed expansion service and developed alongside the backend (but preferably not my be though, I don't do much frontend work).

**Question 4:** We need a nice name for the software... Iquex? Iquexs? Quext? Quexpanse? Better suggestions?
