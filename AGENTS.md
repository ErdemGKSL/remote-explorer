This project going to be a tauri app.

Main window:
Can select a saved project or connect to a new project.

Project page:
When opened it creates a new ssh connection to the destination and cache that connection, this is going to be the main ssh connection to get info about the ssh.
Also cache password, user and host, because in the future we may create more ssh connections.
It will show folders to the user like an normal file explorer. Folder data will be fetched via executing "cd /absolute/route/that/user/currently/in && ls -la" through the main ssh.
