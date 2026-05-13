# BKLibrary SQLite schema

This document lists **every table and column** as reported by SQLite from a live Apple Books BKLibrary database (`BKLibrary-*.sqlite` under the Books app container) as of Version 8.5 (6570). Values were captured with:

```bash
sqlite3 /path/to/BKLibrary-*.sqlite "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name;"
# For each table:
sqlite3 /path/to/BKLibrary-*.sqlite "PRAGMA table_info(<TableName>);"
```

**Source snapshot:** macOS Apple Books, file `BKLibrary-1-091020131601.sqlite`, SQLite library version **3.51.0**, `PRAGMA user_version` = 0, `PRAGMA application_id` = 0. Note that the integer at the end of the sqlite file name appears to be the created timestamp of the database.

Apple may change column sets or types between Books releases. If your file differs, re-run the commands above on your own `BKLibrary*.sqlite`.

**Convention:** Names prefixed with `Z` are the persisted form of Core Data attributes. Tables `Z_METADATA`, `Z_MODELCACHE`, and `Z_PRIMARYKEY` are typical Core Data SQLite store metadata.

---

## Tables (alphabetical)

| Table | Purpose (inferred) |
| --- | --- |
| `ZBCCLOUDSYNCVERSIONS` | Per–data-type cloud sync version / history token bookkeeping |
| `ZBKCOLLECTION` | Reading list / collection metadata |
| `ZBKCOLLECTIONMEMBER` | Membership of an asset in a collection |
| `ZBKJALISCOSTATUS` | Store-related status (Jalisco is Apple’s internal store stack name) |
| `ZBKLIBRARYASSET` | Main library row per book / audiobook / etc. |
| `Z_METADATA` | Core Data store metadata (version, UUID, plist blob) |
| `Z_MODELCACHE` | Core Data model cache blob |
| `Z_PRIMARYKEY` | Core Data per-entity primary-key allocation |

---

## `ZBCCLOUDSYNCVERSIONS`

| Column | Type | `pk` | `notnull` |
| --- | --- | ---: | ---: |
| `Z_PK` | INTEGER | 1 | 0 |
| `Z_ENT` | INTEGER | 0 | 0 |
| `Z_OPT` | INTEGER | 0 | 0 |
| `ZCLOUDVERSION` | INTEGER | 0 | 0 |
| `ZHISTORYTOKENOFFSET` | INTEGER | 0 | 0 |
| `ZLOCALVERSION` | INTEGER | 0 | 0 |
| `ZSYNCVERSION` | INTEGER | 0 | 0 |
| `ZDATATYPE` | VARCHAR | 0 | 0 |
| `ZRAWHISTORYTOKEN` | BLOB | 0 | 0 |

---

## `ZBKCOLLECTION`

| Column | Type | `pk` | `notnull` |
| --- | --- | ---: | ---: |
| `Z_PK` | INTEGER | 1 | 0 |
| `Z_ENT` | INTEGER | 0 | 0 |
| `Z_OPT` | INTEGER | 0 | 0 |
| `ZDELETEDFLAG` | INTEGER | 0 | 0 |
| `ZHIDDEN` | INTEGER | 0 | 0 |
| `ZPLACEHOLDER` | INTEGER | 0 | 0 |
| `ZSORTKEY` | INTEGER | 0 | 0 |
| `ZSORTMODE` | INTEGER | 0 | 0 |
| `ZVIEWMODE` | INTEGER | 0 | 0 |
| `ZLASTMODIFICATION` | TIMESTAMP | 0 | 0 |
| `ZLOCALMODDATE` | TIMESTAMP | 0 | 0 |
| `ZCOLLECTIONID` | VARCHAR | 0 | 0 |
| `ZDETAILS` | VARCHAR | 0 | 0 |
| `ZTITLE` | VARCHAR | 0 | 0 |

---

## `ZBKCOLLECTIONMEMBER`

| Column | Type | `pk` | `notnull` |
| --- | --- | ---: | ---: |
| `Z_PK` | INTEGER | 1 | 0 |
| `Z_ENT` | INTEGER | 0 | 0 |
| `Z_OPT` | INTEGER | 0 | 0 |
| `ZSORTKEY` | INTEGER | 0 | 0 |
| `ZASSET` | INTEGER | 0 | 0 |
| `ZCOLLECTION` | INTEGER | 0 | 0 |
| `ZLOCALMODDATE` | TIMESTAMP | 0 | 0 |
| `ZASSETID` | VARCHAR | 0 | 0 |
| `ZTEMPORARYASSETID` | VARCHAR | 0 | 0 |

---

## `ZBKJALISCOSTATUS`

| Column | Type | `pk` | `notnull` |
| --- | --- | ---: | ---: |
| `Z_PK` | INTEGER | 1 | 0 |
| `Z_ENT` | INTEGER | 0 | 0 |
| `Z_OPT` | INTEGER | 0 | 0 |
| `ZTIMEINTERVALSINCEREFERENCEDATE` | INTEGER | 0 | 0 |
| `ZSTATUS` | VARCHAR | 0 | 0 |
| `ZSTOREID` | VARCHAR | 0 | 0 |

---

## `ZBKLIBRARYASSET`

One row per book, audiobook, PDF, or related item in the user’s library. **Inferred purpose** values are guesses from column names, indexes, and typical Books behavior; Apple does not document this schema publicly.

| Column | Type | `pk` | `notnull` | Inferred purpose |
| --- | --- | ---: | ---: | --- |
| `Z_PK` | INTEGER | 1 | 0 | SQLite row id for this Core Data object (primary key). |
| `Z_ENT` | INTEGER | 0 | 0 | Core Data entity id (which managed object type this row is). |
| `Z_OPT` | INTEGER | 0 | 0 | Optimistic locking / row version for persistence. |
| `ZAUTHORCOUNT` | INTEGER | 0 | 0 | Number of credited authors (for display or validation). |
| `ZCANREDOWNLOAD` | INTEGER | 0 | 0 | Whether the title can be re-downloaded from Apple (rights / availability). |
| `ZCOMBINEDSTATE` | INTEGER | 0 | 0 | Internal combined state flags for the asset (exact meaning unknown). |
| `ZCOMPUTEDRATING` | INTEGER | 0 | 0 | Normalized or store-derived rating value for sorting or UI. |
| `ZCONTENTTYPE` | INTEGER | 0 | 0 | Enum for material type (e.g. ebook vs audiobook vs PDF). |
| `ZDESKTOPSUPPORTLEVEL` | INTEGER | 0 | 0 | How well the title is supported on macOS vs iOS-only features. |
| `ZDIDRUNFORYOUENDOFBOOKEXPERIENCE` | INTEGER | 0 | 0 | Whether Apple’s post-completion / recommendations flow ran for this title. |
| `ZDIDWARNABOUTDESKTOPSUPPORT` | INTEGER | 0 | 0 | User was shown a limited-desktop-support warning. |
| `ZFILESIZE` | INTEGER | 0 | 0 | Approximate on-disk or download size in bytes. |
| `ZFINISHEDDATEKIND` | INTEGER | 0 | 0 | How `ZDATEFINISHED` should be interpreted (e.g. explicit vs inferred). |
| `ZGENERATION` | INTEGER | 0 | 0 | Internal generation counter for sync or content updates. |
| `ZHASRACSUPPORT` | INTEGER | 0 | 0 | Rolled / cloud asset (RAC) support or entitlement flag. |
| `ZHASTOOMANYAUTHORS` | INTEGER | 0 | 0 | Author list exceeded UI limits; show truncated. |
| `ZHASTOOMANYNARRATORS` | INTEGER | 0 | 0 | Narrator list exceeded UI limits; show truncated. |
| `ZISDEVELOPMENT` | INTEGER | 0 | 0 | Pre-release or internal catalog item. |
| `ZISDOWNLOADINGSUPPLEMENTALCONTENT` | INTEGER | 0 | 0 | Supplementary assets (extras) are currently downloading. |
| `ZISEPHEMERAL` | INTEGER | 0 | 0 | Short-lived item (e.g. preview) not treated as owned library content. |
| `ZISEXPLICIT` | INTEGER | 0 | 0 | Marked explicit for parental controls and badges. |
| `ZISFINISHED` | INTEGER | 0 | 0 | User marked reading or listening complete. |
| `ZISHIDDEN` | INTEGER | 0 | 0 | Hidden from default library views but still in the database. |
| `ZISLOCKED` | INTEGER | 0 | 0 | DRM, device limit, or policy lock preventing use. |
| `ZISNEW` | INTEGER | 0 | 0 | Shown as “new” in the library UI. |
| `ZISPROOF` | INTEGER | 0 | 0 | Advance / proof copy rather than retail metadata. |
| `ZISSAMPLE` | INTEGER | 0 | 0 | Sample-only content, not the full book. |
| `ZISSTOREAUDIOBOOK` | INTEGER | 0 | 0 | Purchased or cataloged specifically as a Store audiobook. |
| `ZISSUPPLEMENTALCONTENT` | INTEGER | 0 | 0 | This row is supplemental material tied to another title (see parent FKs). |
| `ZISTRACKEDASRECENT` | INTEGER | 0 | 0 | Eligible for “recently read” or similar surfaces. |
| `ZMAPPEDASSETCONTENTTYPE` | INTEGER | 0 | 0 | Content type after mapping bundled or multi-part assets. |
| `ZMETADATAMIGRATIONVERSION` | INTEGER | 0 | 0 | Which metadata migration pass has been applied to this row. |
| `ZNARRATORCOUNT` | INTEGER | 0 | 0 | Number of audiobook narrators associated with the title. |
| `ZNOTFINISHED` | INTEGER | 0 | 0 | Legacy or auxiliary “not finished” signal (often redundant with `ZISFINISHED`). |
| `ZPAGECOUNT` | INTEGER | 0 | 0 | Page count for reflowable or fixed-layout ebooks. |
| `ZRATING` | INTEGER | 0 | 0 | User star rating or thumbs-style rating stored as an integer. |
| `ZSERIESFILTERMODE` | INTEGER | 0 | 0 | How volumes in a series are filtered in the UI. |
| `ZSERIESISCLOUDONLY` | INTEGER | 0 | 0 | Series exists only in iCloud; not all volumes are local files. |
| `ZSERIESISHIDDEN` | INTEGER | 0 | 0 | Whole series hidden from default views. |
| `ZSERIESISORDERED` | INTEGER | 0 | 0 | Series has an explicit reading order. |
| `ZSERIESNEXTFLAG` | INTEGER | 0 | 0 | “Next in series” or continuation prompt state. |
| `ZSERIESSORTKEY` | INTEGER | 0 | 0 | Integer key for ordering this volume within its series. |
| `ZSERIESSORTMODE` | INTEGER | 0 | 0 | Whether series order is manual, publication date, etc. |
| `ZSORTKEY` | INTEGER | 0 | 0 | Library-wide manual or computed sort position. |
| `ZSTATE` | INTEGER | 0 | 0 | Coarse acquisition or lifecycle state (downloaded, failed, removed, etc.). |
| `ZTASTE` | INTEGER | 0 | 0 | Bits or enum related to personalization / recommendations (“taste”). |
| `ZTASTESYNCEDTOSTORE` | INTEGER | 0 | 0 | Whether taste-related data for this asset was synced to Apple’s services. |
| `ZLOCALONLYSERIESITEMSPARENT` | INTEGER | 0 | 0 | Foreign key to another `ZBKLIBRARYASSET.Z_PK` for a local-only series parent. |
| `ZPURCHASEDANDLOCALPARENT` | INTEGER | 0 | 0 | Foreign key linking a purchased row to a local or merged parent asset. |
| `ZSERIESCONTAINER` | INTEGER | 0 | 0 | Foreign key to the series “container” row in this same table, if any. |
| `ZSUPPLEMENTALCONTENTPARENT` | INTEGER | 0 | 0 | Foreign key to the main book row when this row is supplemental content. |
| `ZASSETDETAILSMODIFICATIONDATE` | TIMESTAMP | 0 | 0 | When extended metadata (not necessarily the file) last changed. |
| `ZBOOKHIGHWATERMARKPROGRESS` | FLOAT | 0 | 0 | Furthest point reached in the book (0.0–1.0); used for resume and UI. |
| `ZBOOKMARKSSERVERMAXMODIFICATIONDATE` | TIMESTAMP | 0 | 0 | Watermark for iCloud bookmarks and annotations sync. |
| `ZCOVERASPECTRATIO` | FLOAT | 0 | 0 | Width-to-height ratio of the cover image. |
| `ZCREATIONDATE` | TIMESTAMP | 0 | 0 | When this library record was created (Core Data / Apple reference date). |
| `ZDATEFINISHED` | TIMESTAMP | 0 | 0 | When the user finished the title (if set). |
| `ZDURATION` | FLOAT | 0 | 0 | Total duration for audiobooks (often seconds; units match Apple’s model). |
| `ZEXPECTEDDATE` | TIMESTAMP | 0 | 0 | Expected availability for pre-orders. |
| `ZFILEONDISKLASTTOUCHDATE` | TIMESTAMP | 0 | 0 | Last time the downloaded file bundle was touched on disk. |
| `ZLASTENGAGEDDATE` | TIMESTAMP | 0 | 0 | Last substantive reading or listening session. |
| `ZLASTOPENDATE` | TIMESTAMP | 0 | 0 | Last time the user opened the title (broader than “engaged”). |
| `ZLOCATIONSERVERMAXMODIFICATIONDATE` | TIMESTAMP | 0 | 0 | Server sync watermark for reading position / locations. |
| `ZMODIFICATIONDATE` | TIMESTAMP | 0 | 0 | Last change to this persisted record. |
| `ZPURCHASEDATE` | TIMESTAMP | 0 | 0 | When the user bought or claimed the title from the Store. |
| `ZREADINGPROGRESS` | FLOAT | 0 | 0 | Current position in the main text or audio (0.0–1.0). |
| `ZRELEASEDATE` | TIMESTAMP | 0 | 0 | Publication or store release date from metadata. |
| `ZUPDATEDATE` | TIMESTAMP | 0 | 0 | Last metadata refresh from the Store or ingestion pipeline. |
| `ZVERSIONNUMBER` | FLOAT | 0 | 0 | Numeric content or bundle version from the publisher or Store. |
| `ZSEQUENCENUMBER` | DECIMAL | 0 | 0 | Series volume index, sometimes fractional for side stories. |
| `ZACCOUNTID` | VARCHAR | 0 | 0 | Apple / media account identifier tied to the purchase. |
| `ZASSETGUID` | VARCHAR | 0 | 0 | Stable UUID string for the asset across devices and restores. |
| `ZASSETID` | VARCHAR | 0 | 0 | Store catalog identifier for the listing (string in current schemas). |
| `ZAUTHOR` | VARCHAR | 0 | 0 | Primary author string shown in the library. |
| `ZBOOKDESCRIPTION` | VARCHAR | 0 | 0 | Synopsis or marketing description text. |
| `ZBOOKMARKSSERVERVERSION` | VARCHAR | 0 | 0 | Version token for bookmark sync with Apple’s servers. |
| `ZCOMMENTS` | VARCHAR | 0 | 0 | Free-form user notes or comments. |
| `ZCOVERURL` | VARCHAR | 0 | 0 | URL for cover artwork. |
| `ZCOVERWRITINGMODE` | VARCHAR | 0 | 0 | Cover or book writing mode (e.g. vertical, RTL) for CJK or similar. |
| `ZDATASOURCEIDENTIFIER` | VARCHAR | 0 | 0 | Which subsystem supplied the metadata (Store, sidecar file, etc.). |
| `ZDOWNLOADEDDSID` | VARCHAR | 0 | 0 | Download or device session identifier (internal; exact semantics unknown). |
| `ZEPUBID` | VARCHAR | 0 | 0 | EPUB-specific identifier when the asset is EPUB-based. |
| `ZFAMILYID` | VARCHAR | 0 | 0 | Store “family” id grouping related editions or regional variants. |
| `ZGENRE` | VARCHAR | 0 | 0 | Primary genre label for display and filtering. |
| `ZGROUPING` | VARCHAR | 0 | 0 | Grouping label, often the series name or anthology title. |
| `ZKIND` | VARCHAR | 0 | 0 | Human-readable media kind (e.g. “ebook”, “audiobook”). |
| `ZLANGUAGE` | VARCHAR | 0 | 0 | Language code or name for the edition. |
| `ZLOCATIONSERVERVERSION` | VARCHAR | 0 | 0 | Server version string for reading-position sync. |
| `ZMAPPEDASSETID` | VARCHAR | 0 | 0 | Alternate or mapped Store id when this row mirrors another listing. |
| `ZPAGEPROGRESSIONDIRECTION` | VARCHAR | 0 | 0 | Page progression direction metadata (LTR vs RTL). |
| `ZPATH` | VARCHAR | 0 | 0 | Path to the on-disk container or primary file inside the sandbox. |
| `ZPERMLINK` | VARCHAR | 0 | 0 | Stable deep link to open or share the title in Apple’s ecosystem. |
| `ZPURCHASEDDSID` | VARCHAR | 0 | 0 | Account or purchase session id associated with the transaction. |
| `ZSEQUENCEDISPLAYNAME` | VARCHAR | 0 | 0 | User-facing series position label (e.g. “Book 2”). |
| `ZSERIESID` | VARCHAR | 0 | 0 | Store series identifier for grouping volumes. |
| `ZSERIESSTACKIDS` | VARCHAR | 0 | 0 | Serialized ids for stacked or merged series relationships. |
| `ZSORTAUTHOR` | VARCHAR | 0 | 0 | Collated author string used for alphabetical sorting. |
| `ZSORTTITLE` | VARCHAR | 0 | 0 | Collated title with articles normalized for sorting. |
| `ZSTOREID` | VARCHAR | 0 | 0 | Store listing id (may duplicate or align with `ZASSETID` depending on era). |
| `ZSTOREPLAYLISTID` | VARCHAR | 0 | 0 | Store playlist or multi-part audiobook id when applicable. |
| `ZTEMPORARYASSETID` | VARCHAR | 0 | 0 | Temporary id before the canonical Store ids are assigned or resolved. |
| `ZTITLE` | VARCHAR | 0 | 0 | Display title of the work. |
| `ZVERSIONNUMBERHUMANREADABLE` | VARCHAR | 0 | 0 | Publisher-facing version label (e.g. “2.1”) as a string. |
| `ZYEAR` | VARCHAR | 0 | 0 | Publication year as stored for display or filtering. |
| `ZURL` | VARCHAR | 0 | 0 | Canonical web or itms-style URL for the product page. |
| `ZAUTHORNAMES` | BLOB | 0 | 0 | Archived structured author list (often plist or keyed archive in a blob). |
| `ZGENRES` | BLOB | 0 | 0 | Archived genre list beyond the single `ZGENRE` string. |
| `ZNARRATORNAMES` | BLOB | 0 | 0 | Archived narrator list for audiobooks. |

---

## `Z_METADATA`

| Column | Type | `pk` | `notnull` |
| --- | --- | ---: | ---: |
| `Z_VERSION` | INTEGER | 1 | 0 |
| `Z_UUID` | VARCHAR(255) | 0 | 0 |
| `Z_PLIST` | BLOB | 0 | 0 |

---

## `Z_MODELCACHE`

| Column | Type | `pk` | `notnull` |
| --- | --- | ---: | ---: |
| `Z_CONTENT` | BLOB | 0 | 0 |

---

## `Z_PRIMARYKEY`

| Column | Type | `pk` | `notnull` |
| --- | --- | ---: | ---: |
| `Z_ENT` | INTEGER | 1 | 0 |
| `Z_NAME` | VARCHAR | 0 | 0 |
| `Z_SUPER` | INTEGER | 0 | 0 |
| `Z_MAX` | INTEGER | 0 | 0 |

---

## Indexes (non–auto-generated)

These were listed by `SELECT name, tbl_name, sql FROM sqlite_master WHERE type='index' AND sql IS NOT NULL ORDER BY tbl_name, name;` on the same file.

| Index | Table | Definition |
| --- | --- | --- |
| `Z_BCCloudSyncVersions_byDataTypeIndex` | `ZBCCLOUDSYNCVERSIONS` | `CREATE INDEX Z_BCCloudSyncVersions_byDataTypeIndex ON ZBCCLOUDSYNCVERSIONS (ZDATATYPE COLLATE BINARY ASC)` |
| `Z_BKCollection_byCollectionIDIndex` | `ZBKCOLLECTION` | `CREATE INDEX Z_BKCollection_byCollectionIDIndex ON ZBKCOLLECTION (ZCOLLECTIONID COLLATE BINARY ASC)` |
| `ZBKCOLLECTIONMEMBER_ZASSET_INDEX` | `ZBKCOLLECTIONMEMBER` | `CREATE INDEX ZBKCOLLECTIONMEMBER_ZASSET_INDEX ON ZBKCOLLECTIONMEMBER (ZASSET)` |
| `ZBKCOLLECTIONMEMBER_ZCOLLECTION_INDEX` | `ZBKCOLLECTIONMEMBER` | `CREATE INDEX ZBKCOLLECTIONMEMBER_ZCOLLECTION_INDEX ON ZBKCOLLECTIONMEMBER (ZCOLLECTION)` |
| `Z_BKCollectionMember_byAssetIDIndex` | `ZBKCOLLECTIONMEMBER` | `CREATE INDEX Z_BKCollectionMember_byAssetIDIndex ON ZBKCOLLECTIONMEMBER (ZASSETID COLLATE BINARY ASC)` |
| `Z_BKJaliscoStatus_byStoreIdIndex` | `ZBKJALISCOSTATUS` | `CREATE INDEX Z_BKJaliscoStatus_byStoreIdIndex ON ZBKJALISCOSTATUS (ZSTOREID COLLATE BINARY ASC)` |
| `ZBKLIBRARYASSET_ZLOCALONLYSERIESITEMSPARENT_INDEX` | `ZBKLIBRARYASSET` | `CREATE INDEX ZBKLIBRARYASSET_ZLOCALONLYSERIESITEMSPARENT_INDEX ON ZBKLIBRARYASSET (ZLOCALONLYSERIESITEMSPARENT)` |
| `ZBKLIBRARYASSET_ZPURCHASEDANDLOCALPARENT_INDEX` | `ZBKLIBRARYASSET` | `CREATE INDEX ZBKLIBRARYASSET_ZPURCHASEDANDLOCALPARENT_INDEX ON ZBKLIBRARYASSET (ZPURCHASEDANDLOCALPARENT)` |
| `ZBKLIBRARYASSET_ZSERIESCONTAINER_INDEX` | `ZBKLIBRARYASSET` | `CREATE INDEX ZBKLIBRARYASSET_ZSERIESCONTAINER_INDEX ON ZBKLIBRARYASSET (ZSERIESCONTAINER)` |
| `ZBKLIBRARYASSET_ZSUPPLEMENTALCONTENTPARENT_INDEX` | `ZBKLIBRARYASSET` | `CREATE INDEX ZBKLIBRARYASSET_ZSUPPLEMENTALCONTENTPARENT_INDEX ON ZBKLIBRARYASSET (ZSUPPLEMENTALCONTENTPARENT)` |
| `Z_BKLibraryAsset_byAssetIDIndex` | `ZBKLIBRARYASSET` | `CREATE INDEX Z_BKLibraryAsset_byAssetIDIndex ON ZBKLIBRARYASSET (ZASSETID COLLATE BINARY ASC)` |
| `Z_BKLibraryAsset_byBookHighWaterMarkProgressIndex` | `ZBKLIBRARYASSET` | `CREATE INDEX Z_BKLibraryAsset_byBookHighWaterMarkProgressIndex ON ZBKLIBRARYASSET (ZBOOKHIGHWATERMARKPROGRESS COLLATE BINARY ASC)` |
| `Z_BKLibraryAsset_bySeriesIDIndex` | `ZBKLIBRARYASSET` | `CREATE INDEX Z_BKLibraryAsset_bySeriesIDIndex ON ZBKLIBRARYASSET (ZSERIESID COLLATE BINARY ASC)` |
| `Z_BKLibraryAsset_bySeriesIsHiddenIndex` | `ZBKLIBRARYASSET` | `CREATE INDEX Z_BKLibraryAsset_bySeriesIsHiddenIndex ON ZBKLIBRARYASSET (ZSERIESISHIDDEN COLLATE BINARY ASC)` |
| `Z_BKLibraryAsset_bySeriesSortKeyIndex` | `ZBKLIBRARYASSET` | `CREATE INDEX Z_BKLibraryAsset_bySeriesSortKeyIndex ON ZBKLIBRARYASSET (ZSERIESSORTKEY COLLATE BINARY ASC)` |
| `Z_BKLibraryAsset_byStorePlaylistIDIndex` | `ZBKLIBRARYASSET` | `CREATE INDEX Z_BKLibraryAsset_byStorePlaylistIDIndex ON ZBKLIBRARYASSET (ZSTOREPLAYLISTID COLLATE BINARY ASC)` |

---

## How to regenerate this document

1. Locate your library file, typically:
   `~/Library/Containers/com.apple.iBooksX/Data/Documents/BKLibrary/BKLibrary-*.sqlite`
2. Prefer opening a **copy** or use read-only access so you do not contend with Books while it has the DB open.
3. Run `PRAGMA table_info(<name>);` for each table name from `sqlite_master`, or dump full DDL with `.schema` in the `sqlite3` shell.
