## MQTT topics:

- gs/{ground_station_id}/jobs: the ground station receives jobs to be executed.
TODO: maybe the gs should just send updates to jobs/ and the id not necessary
- job/{job_id}: the ground station publishes the status of the job to this topic.
- satellite/{satellite_name}/telemetry: the ground station publishes received telemetry frames for the satellite on this topic.

