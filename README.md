# mgdsw

Maxim Games Discord Server Web Site. This will be a server for deploying a web server dedicated to the Discord server created by Maxim.

## Command that usefully during developing

For build final fat JAR use
```bash
./gradlew shadowjar
```

For run tests use
```bash
./gradlew test
```

## Requirements for start

- JDK 25

The build fat JAR is created under `build/libs/`.

So for run server using fat jar use command. 
```bash
java -jar build/libs/mgdsw-pre-alpha-2026-10-19-all.jar
```