DOIT_CONFIG = {"default_tasks": ["pip"]}


def task_pip():
    return {
        "actions": [
            "pip-compile --output-file=requirements.txt requirements.in",
            "pip-sync",
            "sed -i -e '/macfsevents/d' requirements.txt",
            "sed -i -e '/pyinotify/d' requirements.txt",
        ],
        "file_dep": ["requirements.in", "dodo.py"],
        "targets": ["requirements.txt"],
    }
