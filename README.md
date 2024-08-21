# WikiJS Module | Meilisearch

## Description

This module is a plugin for the [WikiJS](https://wiki.js.org/) software. It allows you to use the [Meilisearch](https://meilisearch.com/) search engine to index and search your wiki content.

## Pre-requisites

1. A running instance of Meilisearch
2. A running instance of WikiJS

## Installation

1. Navigate to the latest release of the module on the [releases page](https://github.com/mbround18/wikijs-module-meilisearch/releases).
2. Download the `meilisearch.zip` file.
3. Extract the contents of the zip file into the `/wiki/server/modules/meilisearch` directory.
4. Restart your WikiJS server.
5. Navigate to your admin panel
6. Click on the `Search` tab
7. Select `Meilisearch` from the dropdown
8. Enter the URL of your Meilisearch server (e.g. `http://localhost:7700`)
9. Change the API key
10. Click `Apply`

> **Note:** If you do not see a green success message, try to apply again and then check the logs.
