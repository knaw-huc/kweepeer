kweepeer(5)

# NAME

kweepeer - a generic webservice for interactive query expansion, expansion is provided via various modules

# SYNTAX

A kweepeer instantance is configured via a single configuration file in *TOML*
syntax. See https://toml.io/en/ for generic information about TOML. Kweepeer
modules are defined as a so-called array of tables (between double square
brackets). 

# MODULES

The following module types can be defined, assuming kweepeer was
compiled with support for all. We will discuss each in a subsection:

*analiticcl* 
	This module uses a library called analiticcl to identify lexical syntactic variants
	for the query terms. This module uses anagram hashing to find all terms with
	a given edit-distance and incorporates various other techniques to identify 
	the best spelling variants. The input for the module is typically a curated lexicon.
	See section _ANALITICCL_.

*finalfusion*
	This module finds similar terms via word embeddings and vector comparison.
	The input is a word embedding model in finalfusion format as computed by
	_finalfrontier_(1) . See section _FINALFUSION_.

*fst*
	This is a Finite State Transducer to identify lexical syntactic variants for the query
	times. It finds all terms within a given edit-distance (Levenshtein). The
	input is a curated or extracted lexicon. See section _FST_.

*lookup*
	This is a simple module that stores expansions explicitly in a hash-map. It 
	maps a key term to one or more expansion terms. The input for this is a TSV
	file. See section _LOOKUP_.

All modules take the following mandatory parameters:

*id*
	The identifier of the module (alphanumeric without spaces, periods, slashes,
	ampersands; lowercase recommended).
*name*
	A human-readable name for the module

## ANALITICCL

The analiticcl module takes the following parameters in addition to the common
parameters:

*alphabet* (mandatory)
	Path to the alphabet file. See
	https://github.com/proycon/analiticcl?tab=readme-ov-file#alphabet-file for
	the syntax of that file.

*lexicons* (mandatory)
	Array of tables. Each lexicon takes a *filename* parameter pointing to a lexicon
	file. A lexicon may contain frequency information. See
	https://github.com/proycon/analiticcl?tab=readme-ov-file#lexicon-file .
	Furthermore, a additional *params* parameter can be set, containing extra
	parameters (key/value pairs) on how to parse the lexicon file.


The following example illustrates a simple configuration for an
analiticcl module:

```
[[analiticcl]]
id = "nl_voc_analiticcl"
name = "NL VOC groundtruth lexicon"
alphabet = "simple.alphabet.tsv"

[[analiticcl.lexicons]]
filename = "nl_voc.tsv"
```


## FINALFUSION

The finalfusion module takes the following parameters in addition to the common
parameters:

*file* (mandatory)
	Path to a the finalfusion word embedding model.

*k* (optional, default 10)
	The number of results to return

This module is powered by finalfrontier: https://finalfusion.github.io/finalfrontier
Here is a quick example of generating a finalfusion model using _finalfrontier_(1):

```
$ finalfrontier skipgram --dims 300 --model structgram --epochs 10 --mincount 10 \
  --threads 16 tokenised-corpus.txt embeddings.fifu
```

The following example illustrates a simple configuration for a
finalfusion module:

```
[[finalfusion]]
id = "embeddings"
name = "Word Embeddings"
file = "embeddings.fifu"
k = 10
```


## FST

The fst module takes the following parameters in addition to the common
parameters:

*file* (string, mandatory)
	Path to a lexicon with one term per line, file may be tab-separated-values, everything except
	the first column is ignored.

*distance* (int, optional)
	The default levenshtein distance, can be overriden at runtime.

*sorted* (bool, optional, default false)
    Is the lexicon already sorted lexographically? If it is, setting this to true improves loading time/memory consumption

*skipfirstline* (bool, optional, default false)
    Set this if the first line is a header

*casesensitive* (bool, optional, default false)
    Do case sensitive lookups

The following example illustrates a simple configuration for a
lookup module:

```
[[fst]]
id = "nl_voc_fst"
name = "NL VOC groundtruth lexicon"
file = "nl_voc.tsv"
distance = 2
```

This module is powered by BurntSushi's fst module: https://crates.io/crates/fst

## LOOKUP

The lookup module takes the following parameters in addition to the common
parameters:

*file* (mandatory)
	Path to the lexicon file.

*delimiter* (optional)
	Primary delimiter; between term key and variants (defaults to tab)	

*delimiter2* (optional)
	Secondary delimiter; between each variant (defaults to tab)	

*skipfirstline* (bool, optional, default false)
    Set this if the first line is a header

*casesensitive* (bool, optional, default false)
    Do case sensitive lookups

*allow_numeric* (bool, optional, default false)
    Allow numeric fields, otherwise they will be ignored (which is useful to filter out frequency/score information from input files)

The following example illustrates a simple configuration for a
lookup module:

```
[[lookup]]
id = "inthislex"
name = "INT Historisch Lexicon"
file = "int_historisch_lexicon_variants.tsv"
```

# SOURCE & CONTRIBUTE

See https://github.com/knaw-huc/kweepeer

# LICENSE

GNU Affero General Public License v3

# SEE ALSO

*kweepeer*(1) for configuration instructions.
*finalfrontier*(1)

# AUTHORS

Maarten van Gompel <proycon@anaproy.nl>, Digital Infrastructure, KNAW Humanities Cluster

# ACKNOWLEDGEMENT

Developed in the scope of the Globalise Project <https://globalise.huygens.knaw.nl/>
