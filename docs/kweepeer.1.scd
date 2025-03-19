kweepeer(1)

# NAME

kweepeer - a generic webservice for interactive query expansion, expansion is provided via various modules

# SYNOPSIS

*kweepeer* [OPTIONS]

# OPTIONS

*--debug*
	Output logging info on incoming requests
*-b*, *--bind* _host_:_port_
	The host and port to bind to, defaults to 127.0.0.1:8080	
*-c*, *--config* _file_
	The configuration file, this should be a _toml_ file. See *kweepeer*(5) for
	configuration instructions.
*--version*
	Print program version and exit.
*-h* *--help*
	Print command line argument help.

# WEB API

This starts an HTTP webservice with the following endpoints:

*GET* _/_
	Main entrypoint. Use parameter *q* to pass a query in Lucene syntax.
	Use parameters *include* or *exclude* to include/exclude modules by ID.
	They take a comma separated list. Response will be JSON. 
*GET* _/modules_
	Lists all available modules. Responds in JSON.
*GET* _/swagger-ui_
	Interactive swagger/OpenAPI web interface showing the Web API specification
*GET* _/api-doc/openapi.json_
	OpenAPI specification

# SOURCE & CONTRIBUTE

See https://github.com/knaw-huc/kweepeer

# LICENSE

GNU Affero General Public License v3

# SEE ALSO

*kweepeer*(5) for configuration instructions.

# AUTHORS

Maarten van Gompel <proycon@anaproy.nl>, Digital Infrastructure, KNAW Humanities Cluster

# ACKNOWLEDGEMENT

Developed in the scope of the Globalise Project <https://globalise.huygens.knaw.nl/>
