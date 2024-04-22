#!/bin/env python3
##
#  Asmov Git Repository Template 
#  Copyright (C) 2024 Asmov LLC
#
#  This program is free software: you can redistribute it and/or modify
#  it under the terms of the GNU General Public License as published by
#  the Free Software Foundation, either version 3 of the License, or
#  (at your option) any later version.
#
#  This program is distributed in the hope that it will be useful,
#  but WITHOUT ANY WARRANTY; without even the implied warranty of
#  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#  GNU General Public License for more details.
# 
#  You should have received a copy of the GNU General Public License
#  along with this program.  If not, see <https://www.gnu.org/licenses/>.
##

import os;
import os.path as path;
import shutil;
import glob;
import sys;

SCRIPT_NAME = __file__
PROJECT_DIR = path.realpath(path.dirname(__file__))

def print_usage():
    print("usage: {SCRIPT_NAME} <config | create | help>")
    print("Recursively copies files from a template directory to a target directory, replacing template variables with configured values.")

def print_help_config():
    print("usage: [STDIN: JSON config defaults] | ] {SCRIPT_NAME} config <template directory> [target directory]")
    print("Interactively creates and outputs a JSON config for a template copy. This is typically one-time use. Optionally uses JSON piped from STDIN to filly in some initial values")
    print("If [target directory] is specified, then `{SCRIPT_NAME} create` will be ran using the config.")

def print_help_create():
    print("usage 1: <STDIN: JSON config> | {SCRIPT_NAME} create [options] <template directory> <target directory>")
    print("usage 2: {SCRIPT_NAME} create <JSON config filepath> <template directory> <target directory>")
    print("Recursively copies files from a template directory to a target directory, replacing template variables with configured values.")
    print("Options:")
    print("--exists <strategy>: If <target directory> exists, the following strategies can be used:")
    print("    git :: Retains the .git directory, deletes everything else. Cannot be used if there are uncomitted changes.")
    print("    replace :: Copies files and directories over the existing ones. Creates a backup of the original.")
    print("    delete :: Deletes the directory contents. Creates a backup of the original.")
    print("--tmpdir <path>: Where backups are stored. Defaults to ~/tmp or /tmp if not specified.")

def main():
    argc = len(sys.argv)
    if argc < 3:
        print_usage()
        sys.exit(1)

    match sys.argv[1]:
        case 'config':
            cmd = CMD_CONFIG
            
        case 'create':
            cmd= CMD_CREATE
        case _:
            print_usage()
            sys.exit(1)
    
if __name__ == '__main__':
    main()
