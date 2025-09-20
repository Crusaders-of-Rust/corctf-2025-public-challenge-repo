#! /bin/bash
docker build . -t msfrognymize2
docker run -p 8443:8443 msfrognymize2
