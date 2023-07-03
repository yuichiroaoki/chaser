#!/bin/bash

# Start Neo4j
docker run -d \
    --publish=7474:7474 --publish=7687:7687 \
	--name neo4j-container \
    --volume=$HOME/neo4j/data:/data \
	--env NEO4J_AUTH=neo4j/testtest \
    neo4j