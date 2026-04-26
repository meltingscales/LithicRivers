# Stage Artifacts Legal

New-Item -Path "./artifacts" -ItemType Directory
Copy-Item -Path "CHANGELOG.txt" -Destination "./artifacts/"
Copy-Item -Path "LICENSE" -Destination "./artifacts/"
Copy-Item -Path "THIRD-PARTY-NOTICES.txt" -Destination "./artifacts/"
