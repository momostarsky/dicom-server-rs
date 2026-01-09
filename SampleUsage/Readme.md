### how to deploy dicom-server-rs

To deploy `dicom-server-rs`, follow these steps:

- Git Clone Project from GitHub

- ***cargo build --release***

- copy wado-server wado-storescp wado-consumer wado-webworker to $USER_HOME/dicom-server-rs

- create  application.prod.json from Sample File : application.Sample.json

- check docker-service state 

Deploy Result is like this:

![Deploy Structure](deploy-struct.png)

![Docker State](docker-state.png) 

![Send Dicom File To WADO-STORESCP](dicom-scu-tools.png)

![Start WADO-SERVER](wado-server-api.png)

![WADO-SERVER API Test with Postman](wado-server.png)
