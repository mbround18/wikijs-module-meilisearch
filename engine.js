// noinspection JSUnusedGlobalSymbols

/**
 * @typedef {import("./pkg/millisearch")} wasm
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
const wasm = require("./millisearch.js");

/**
 * @type {WikiSearchEngine}
 */
let searchEngine;

/**
 * @type {Console}
 */
let logger = WIKI.logger;

/**
 *
 * @param millisearchHost
 * @param millisearchApiKey
 * @param indexName
 * @param timeout
 * @returns {Promise<WikiSearchEngine>}
 */
async function getSearchEngine({
  millisearchHost,
  millisearchApiKey,
  indexName,
  timeout,
}) {
  if (!wasm.WikiSearchEngine) {
    throw new Error(
      `(SEARCH/MILLISEARCH) WikiSearchEngine is not defined. Make sure to add the search engine to your dependencies.`,
    );
  }

  if (!searchEngine) {
    logger.info(
      `(SEARCH/MILLISEARCH) Configuring search engine with host: ${millisearchHost}, index: ${indexName}`,
    );
    searchEngine = await new wasm.WikiSearchEngine(
      millisearchHost || "http://millisearch:7700",
      millisearchApiKey || "demo",
      indexName || "wiki_index",
      BigInt(timeout || 5000),
    );
  }

  return searchEngine;
}

module.exports = {
  /**
   * ACTIVATE
   */
  async activate() {
    logger.info(`(SEARCH/MILLISEARCH) Activating search engine...`);
    const engine = await getSearchEngine(this.config);
    // log all methods attached to engine
    logger.info(`(SEARCH/MILLISEARCH) Engine methods: ${Object.keys(engine)}`);
    await engine.activated();
    logger.info(`(SEARCH/MILLISEARCH) Search engine activated.`);
  },
  /**
   * DEACTIVATE
   */
  async deactivate() {
    logger.info(`(SEARCH/MILLISEARCH) Deactivating search engine...`);
    //     const engine = await getSearchEngine(this.config);
    // await engine.deactivated();
    logger.info(`(SEARCH/MILLISEARCH) Search engine deactivated.`);
  },
  /**
   * INIT
   */
  async init() {
    logger.info(`(SEARCH/MILLISEARCH) Initializing search engine...`);
    const engine = await getSearchEngine(this.config);
    await engine.healthcheck();
    logger.info(`(SEARCH/MILLISEARCH) Search engine initialized.`);
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
  async query(q, opts) {
    try {
      logger.info(
        `(SEARCH/MILLISEARCH) Querying search engine with query: ${q}`,
      );
      const engine = await getSearchEngine(this.config);
      const results = (await engine.query(q)) || [];
      logger.info(
        `(SEARCH/MILLISEARCH) Query returned ${results.length} results.`,
      );
      // locale is required for search but nowhere else. So we need to map it to localCode
      results.results = (results.results || []).map(
        /**
         *
         * @param {WikiPage} s
         * @returns {SearchResultsResponse}
         */
        (s) => {
          let locale = s.localeCode;
          delete s.localeCode;
          // Not being found here is expected.
          s.locale = locale;
          return s;
        },
      );
      return results;
    } catch (err) {
      logger.warn(
        `(SEARCH/MILLISEARCH) Query failed with error: ${err.message}`,
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
  async suggest(q, opts) {
    try {
      logger.info(`(SEARCH/MILLISEARCH) Fetching suggestions for query: ${q}`);
      const engine = await getSearchEngine(this.config);
      const suggestions = await engine.suggest(q, opts);
      logger.info(`(SEARCH/MILLISEARCH) Suggestions fetched successfully.`);

      return suggestions;
    } catch (err) {
      logger.warn(
        `(SEARCH/MILLISEARCH) Suggest failed with error: ${err.message}`,
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
    logger.info(
      `(SEARCH/MILLISEARCH) Creating search index for page: ${page.path}`,
    );
    const engine = await getSearchEngine(this.config);
    await engine.created(page);
    logger.info(
      `(SEARCH/MILLISEARCH) Search index created for page: ${page.path}`,
    );
  },
  /**
   * UPDATE
   *
   * @param {Object} page Page to update
   */
  async updated(page) {
    logger.info(
      `(SEARCH/MILLISEARCH) Updating search index for page: ${page.path}`,
    );
    const engine = await getSearchEngine(this.config);
    await engine.updated(page);
    logger.info(
      `(SEARCH/MILLISEARCH) Search index updated for page: ${page.path}`,
    );
  },
  /**
   * DELETE
   *
   * @param {Object} page Page to delete
   */
  async deleted(page) {
    logger.info(
      `(SEARCH/MILLISEARCH) Deleting search index for page: ${page.path}`,
    );
    const engine = await getSearchEngine(this.config);
    await engine.deleted(page);
    logger.info(
      `(SEARCH/MILLISEARCH) Search index deleted for page: ${page.path}`,
    );
  },
  /**
   * RENAME
   *
   * @param {Object} page Page to rename
   */
  async renamed(page) {
    logger.info(
      `(SEARCH/MILLISEARCH) Renaming search index for page: ${page.path}`,
    );
    const engine = await getSearchEngine(this.config);
    await engine.updated(page);
    logger.info(
      `(SEARCH/MILLISEARCH) Search index renamed for page: ${page.destinationPath}`,
    );
  },
  /**
   * REBUILD INDEX
   */
  async rebuild() {
    logger.info(`(SEARCH/MILLISEARCH) Rebuilding entire search index...`);
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
          { realId: "id" },
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
        console.log(
          `Processing page with ID ${JSON.stringify(row, null, 2)}...`,
        );
        try {
          // Perform delete operation
          await engine.deleted(row);
          // Perform create operation
          await engine.created(row);
          console.log(`Page with ID ${row.id} processed successfully.`);
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

      logger.info(`(SEARCH/MILLISEARCH) Search index rebuilt successfully.`);
    } catch (err) {
      logger.error(
        `(SEARCH/MILLISEARCH) Error rebuilding search index: ${err}`,
      );
    }
  },
};
