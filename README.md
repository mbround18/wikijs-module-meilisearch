<div style="text-align:center;">
   <img src="docs/assets/logo.png" alt="logo" width="200" style="height:auto;" />
</div>

# WikiJS Module | Meilisearch

## Table of Contents

1. [Description](#description)
2. [Pre-requisites](#pre-requisites)
3. [Installation](#installation)
4. [Security Hardening & API Key Usage](#security-hardening--api-key-usage)

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

## Security Hardening & API Key Usage

For production environments, it is strongly recommended to use a restricted Meilisearch API key instead of the master key. The master key grants full access to all indexes and administrative actions, which is not safe for most deployments.

### Why use a restricted API key?

- Limits access to only the necessary actions (search, add, update, delete documents)
- Restricts access to only the configured index (e.g., `wiki_index`)
- Reduces risk if the key is leaked or misused

### How to create a restricted API key in Meilisearch

1. Log in to your Meilisearch dashboard or use the Meilisearch API.
2. Go to the **API Keys** section.
3. Click **Create an API Key**.
4. Set the following options:
   - **Actions**: `search`, `documents.add`, `documents.get`, `documents.update`, `documents.delete`
   - **Indexes**: `wiki_index` (or your configured index name)
   - **Description**: e.g., `WikiJS Search Key`
   - **Expires At**: (optional, for key rotation)
5. Save the key and copy the generated value.
6. In the WikiJS Meilisearch module settings, use this API key for the `Meilisearch: API Key` field instead of the master key.

> **Note:** The master key should only be used for initial setup or administrative tasks. For daily operation, always use a restricted API key.
