// noinspection JSUnusedGlobalSymbols

/**
 * @typedef {import("./pkg/meilisearch")} wasm
 * @typedef {wasm.WikiSearchEngine} WikiSearchEngine
 */

/**
 * Represents a wiki page with various properties.
 *
 * @typedef {Object} WikiPage
 * @property {number} id - The unique identifier of the wiki page.
 * @property {string} path - The path or URL of the wiki page.
 * @property {string} hash - The hash value associated with the wiki page.
 * @property {string} title - The title of the wiki page.
 * @property {string} description - The description of the wiki page.
 * @property {boolean} isPrivate - Indicates whether the wiki page is private or not.
 * @property {boolean} isPublished - Indicates whether the wiki page is published or not.
 * @property {string} content - The content of the wiki page.
 * @property {string} contentType - The content type of the wiki page.
 * @property {string} createdAt - The date and time when the wiki page was created.
 * @property {string} updatedAt - The date and time when the wiki page was last updated.
 * @property {string} editorKey - The key associated with the editor of the wiki page.
 * @property {string} localeCode - The locale code of the wiki page.
 * @property {number} authorId - The unique identifier of the wiki page author.
 * @property {number} creatorId - The unique identifier of the wiki page creator.
 */

/**
 * @type {wasm}
 */
const wasm = require("./pkg/meilisearch");

/**
 * @type {WikiSearchEngine}
 */
let searchEngine;

/**
 * @type {Console}
 */
let logger = WIKI?.logger || console;

/**
 *
 * @param meilisearchHost
 * @param meilisearchApiKey
 * @param indexName
 * @param timeout
 * @returns {Promise<WikiSearchEngine>}
 */
async function getSearchEngine({
  meilisearchHost,
  meilisearchMasterKey,
  indexName,
  timeout,
}) {
  // Allow overriding init options via environment variables when process is defined
  const resolveConfig = (cfg) => {
    try {
      if (typeof process !== "undefined" && process && process.env) {
        const env = process.env;
        return {
          meilisearchHost: env.MEILISEARCH_HOST || cfg.meilisearchHost,
          meilisearchMasterKey:
            env.MEILISEARCH_MASTER_KEY || cfg.meilisearchMasterKey,
          indexName: env.MEILISEARCH_INDEX_NAME || cfg.indexName,
          timeout: env.MEILISEARCH_TIMEOUT
            ? Number(env.MEILISEARCH_TIMEOUT)
            : cfg.timeout,
        };
      }
    } catch (_) {}
    return cfg;
  };

  const resolved = resolveConfig({
    meilisearchHost,
    meilisearchMasterKey,
    indexName,
    timeout,
  });

  if (!wasm.WikiSearchEngine) {
    throw new Error(
      `(SEARCH/MEILISEARCH) WikiSearchEngine is not defined. Make sure to add the search engine to your dependencies.`
    );
  }

  if (!searchEngine) {
    const safeKey = (resolved.meilisearchMasterKey || "").replace(
      /.(?=.{4})/g,
      "*"
    );
    logger.info(
      `(SEARCH/MEILISEARCH) Initializing engine with host=${
        resolved.meilisearchHost || "http://meilisearch:7700"
      }, index=${resolved.indexName || "wiki_index"}, timeout=${
        resolved.timeout || 5000
      }, key=${safeKey}`
    );
    searchEngine = await new wasm.WikiSearchEngine(
      resolved.meilisearchHost || "http://meilisearch:7700",
      resolved.meilisearchMasterKey || "demo",
      resolved.indexName || "wiki_index",
      BigInt(resolved.timeout || 5000)
    );
  }

  return searchEngine;
}

function rejectIfIsPrivateAndNotPublished(page, action) {
  if (!page.isPublished) {
    logger.warn(
      `(SEARCH/MEILISEARCH) SKIPPING: Page with path ${page.path} is not published.`
    );
    return Promise.resolve();
  } else if (page.isPrivate) {
    logger.warn(
      `(SEARCH/MEILISEARCH) SKIPPING: Page with path ${page.path} is private.`
    );
    return Promise.resolve();
  } else {
    return action(page);
  }
}

module.exports = {
  /**
   * ACTIVATE
   */
  async activate(opts = {}) {
    logger.log(`(SEARCH/MEILISEARCH) Activating search engine...`, opts);
    const engine = await getSearchEngine(this.config);
    logger.info(`(SEARCH/MEILISEARCH) Engine methods: ${Object.keys(engine)}`);
    await engine.activated();
    logger.info(`(SEARCH/MEILISEARCH) Search engine activated.`);
  },
  /**
   * DEACTIVATE
   */
  async deactivate() {
    logger.info(`(SEARCH/MEILISEARCH) Deactivating search engine...`);
    //     const engine = await getSearchEngine(this.config);
    // await engine.deactivated();
    logger.info(`(SEARCH/MEILISEARCH) Search engine deactivated.`);
  },
  /**
   * INIT
   */
  async init() {
    logger.info(`(SEARCH/MEILISEARCH) Initializing search engine...`);
    const engine = await getSearchEngine(this.config);
    await engine.healthcheck();
    logger.info(`(SEARCH/MEILISEARCH) Search engine initialized.`);
  },

  /**
   * @typedef {WikiPage} SearchResultsResponse
   * @property {string} locale - This is required mutation from localeCode to locale only for search.
   */

  /**
   * Represents the response structure for a page search query.
   *
   * @typedef {Object} PageSearchResponse
   * @property {string[]} suggestions - A list of suggested search terms based on the query.
   * @property {SearchResultsResponse[]} results - The list of results returned from the search query.
   * @property {number} total_hits - The total number of hits (results) found for the query.
   */

  /**
   * Queries the search engine with the specified query string and options.
   *
   * This function logs the query process, retrieves search results, and formats
   * the results to include locale information. It returns a structured response
   * containing the search results and other relevant data.
   *
   * @async
   * @param {string} q - The search query string to be executed.
   * @param {Object} opts - Optional parameters for the search query.
   * @returns {Promise<PageSearchResponse>} A promise that resolves to a PageSearchResponse object.
   *
   * @throws {Error} Throws an error if the query fails.
   */
  async query(q, opts = {}, ...args) {
    try {
      const engine = await getSearchEngine(this.config);
      const results = (await engine.query(q)) || [];
      logger.info(`[DEBUG] Query results object:`, results);
      logger.info(
        `(SEARCH/MEILISEARCH) Query returned ${
          results && results.results && Array.isArray(results.results)
            ? results.results.length
            : 0
        } results.`
      );
      results.results = (results.results || []).map((s) => {
        if (s.localeCode) {
          s.locale = s.localeCode;
          delete s.localeCode;
        }
        return s;
      });
      return results;
    } catch (err) {
      logger.warn(
        `(SEARCH/MEILISEARCH) Query failed with error: ${err.message}`
      );
      throw err;
    }
  },
  /**
   * SUGGEST
   *
   * @param {String} q Query
   * @param {Object} opts Additional options
   */
  async suggest(q, opts = {}) {
    logger.info(`[DEBUG] Suggest called with query: ${q} and opts:`, opts);
    try {
      logger.info(`(SEARCH/MEILISEARCH) Fetching suggestions for query: ${q}`);
      const engine = await getSearchEngine(this.config);
      const suggestions = await engine.suggest(q);
      logger.info(`(SEARCH/MEILISEARCH) Suggestions fetched successfully.`);
      return suggestions;
    } catch (err) {
      logger.warn(
        `(SEARCH/MEILISEARCH) Suggest failed with error: ${err.message}`
      );
      throw err;
    }
  },
  /**
   * CREATE
   *
   * @param {Object} page Page to create
   */
  async created(page) {
    await rejectIfIsPrivateAndNotPublished(page, async (page) => {
      logger.info(
        `(SEARCH/MEILISEARCH) Creating search index for page: ${page.path}`
      );
      const engine = await getSearchEngine(this.config);
      await engine.created(page);
      logger.info(
        `(SEARCH/MEILISEARCH) Search index created for page: ${page.path}`
      );
    });
  },
  /**
   * UPDATE
   *
   * @param {Object} page Page to update
   */
  async updated(page) {
    await rejectIfIsPrivateAndNotPublished(page, async (page) => {
      logger.info(
        `(SEARCH/MEILISEARCH) Updating search index for page: ${page.path}`
      );
      const engine = await getSearchEngine(this.config);
      await engine.updated(page);
      logger.info(
        `(SEARCH/MEILISEARCH) Search index updated for page: ${page.path}`
      );
    });
  },
  /**
   * DELETE
   *
   * @param {Object} page Page to delete
   */
  async deleted(page) {
    await rejectIfIsPrivateAndNotPublished(page, async (page) => {
      logger.info(
        `(SEARCH/MEILISEARCH) Deleting search index for page: ${page.path}`
      );
      const engine = await getSearchEngine(this.config);
      await engine.deleted(page);
      logger.info(
        `(SEARCH/MEILISEARCH) Search index deleted for page: ${page.path}`
      );
    });
  },
  /**
   * RENAME
   *
   * @param {Object} page Page to rename
   */
  async renamed(page) {
    await rejectIfIsPrivateAndNotPublished(page, async (page) => {
      logger.info(
        `(SEARCH/MEILISEARCH) Renaming search index for page: ${page.path}`
      );
      const engine = await getSearchEngine(this.config);
      await engine.updated(page);
      logger.info(
        `(SEARCH/MEILISEARCH) Search index renamed for page: ${page.destinationPath}`
      );
    });
  },
  /**
   * REBUILD INDEX
   */
  async rebuild() {
    logger.info(`(SEARCH/MEILISEARCH) Rebuilding entire search index...`);
    const engine = await getSearchEngine(this.config);

    try {
      const stream = WIKI.models.knex
        .column(
          { id: "hash" },
          "path",
          { locale: "localeCode" },
          "title",
          "description",
          "hash",
          "isPrivate",
          "isPublished",
          "content",
          "contentType",
          "createdAt",
          "updatedAt",
          "editorKey",
          "authorId",
          "creatorId",
          "localeCode",
          { realId: "id" }
        )
        .select()
        .from("pages")
        .where({
          isPublished: true,
          isPrivate: false,
        })
        .stream();

      // Use a promise to handle the streaming process
      const processRow = async (row) => {
        row.id = row.realId;
        try {
          // Perform delete operation
          await engine.deleted(row);
          // Perform create operation
          const results = await engine.created(row);
          if (logger.debug)
            logger.debug(`[DEBUG] Results from engine.created:`, results);
        } catch (err) {
          console.error(`Error processing page with ID ${row.id}: ${err}`);
        }
      };

      // Listen for data events and process each row
      stream.on("data", (row) => {
        processRow(row);
      });

      // Wait for the stream to finish
      await new Promise((resolve, reject) => {
        stream.on("end", () => {
          console.log("All pages processed.");
          resolve();
        });
        stream.on("error", (err) => {
          console.error(`Stream error: ${err}`);
          reject(err);
        });
      });

      logger.info(`(SEARCH/MEILISEARCH) Search index rebuilt successfully.`);
    } catch (err) {
      logger.error(
        `(SEARCH/MEILISEARCH) Error rebuilding search index: ${err}`
      );
    }
  },
};
