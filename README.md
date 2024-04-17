# Background

Run a command line application in background, while displaying an icon in the system tray.

## Usage

First, write a configuration file for the application you want to run in background. The configuration file is a JSON file with the following format:

```json
{
    "command": "command to run",
    "args": ["list", "of", "arguments"],
    "icon": "path to icon file",
}
```

Then, start the application in background by running the following command:

```bash
background path/to/config.json
```

Now, the application will run in background, and you will see an icon in the system tray. You can click on the icon to open a menu with options to restart or stop the application.

## TODO

- [ ] Check stdout and stderr of the application
- [ ] Multi-platform support (currently only works on Windows)

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
