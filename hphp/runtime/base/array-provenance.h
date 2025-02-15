/*
  +----------------------------------------------------------------------+
  | HipHop for PHP                                                       |
  +----------------------------------------------------------------------+
  | Copyright (c) 2010-present Facebook, Inc. (http://www.facebook.com)  |
  +----------------------------------------------------------------------+
  | This source file is subject to version 3.01 of the PHP license,      |
  | that is bundled with this package in the file LICENSE, and is        |
  | available through the world-wide-web at the following url:           |
  | http://www.php.net/license/3_01.txt                                  |
  | If you did not receive a copy of the PHP license and are unable to   |
  | obtain it through the world-wide-web, please send a note to          |
  | license@php.net so we can mail you a copy immediately.               |
  +----------------------------------------------------------------------+
*/

#ifndef incl_HPHP_ARRAY_PROVENANCE_H
#define incl_HPHP_ARRAY_PROVENANCE_H

#include "hphp/runtime/base/runtime-option.h"
#include "hphp/runtime/base/typed-value.h"
#include "hphp/runtime/base/types.h"

#include "hphp/util/low-ptr.h"
#include "hphp/util/rds-local.h"

#include <folly/Format.h>
#include <folly/Optional.h>

namespace HPHP {

struct APCArray;
struct ArrayData;
struct StringData;
struct c_WaitableWaitHandle;

namespace arrprov {

///////////////////////////////////////////////////////////////////////////////

/*
 * A provenance annotation
 *
 * We need to store the filename and line since when assembling units, we
 * don't necessarily have the final Unit allocated yet. It may be faster to
 * make this a tagged union or store a different Tag type for static arrays
 */
struct Tag {
  Tag() = default;
  Tag(const StringData* filename, int line)
    : m_filename(filename)
    , m_line(line) {
    assertx(m_filename);
  }

  const StringData* filename() const { return m_filename; }
  int line() const { return m_line; }

  bool operator==(const Tag& other) const {
    return m_filename == other.m_filename &&
           m_line == other.m_line;
  }
  bool operator!=(const Tag& other) const { return !(*this == other); }

  std::string toString() const;

private:
  const StringData* m_filename{nullptr};
  int m_line{0};
};

/*
 * This is a separate struct so it can live in RDS and not be GC scanned--the
 * actual RDS-local handle is kept in the implementation
 */
struct ArrayProvenanceTable {
  /* The table itself -- allocated in general heap */
  folly::F14FastMap<const void*, Tag> tags;

  /*
   * We never dereference ArrayData*s from this table--so it's safe for the GC
   * to ignore them in this table
   */
  TYPE_SCAN_IGNORE_FIELD(tags);
};

///////////////////////////////////////////////////////////////////////////////

/*
 * Create a tag based on the current PC and unit.
 *
 * Attempts to sync VM regs and returns folly::none on failure.
 */
folly::Optional<Tag> tagFromPC();

/*
 * RAII struct for modifying the behavior of tagFromPC().
 *
 * When this is in effect, we backtrace from `wh` instead of vmfp().
 */
struct TagOverride {
  explicit TagOverride(c_WaitableWaitHandle* wh);
  ~TagOverride();

private:
  c_WaitableWaitHandle* m_saved_wh;
};

/*
 * Whether `a` or `tv` admits a provenance tag---i.e., whether it's either a
 * vec or a dict.
 */
bool arrayWantsTag(const ArrayData* a);
bool arrayWantsTag(const APCArray* a);

/*
 * Get the provenance tag for `a`.
 */
folly::Optional<Tag> getTag(const ArrayData* a);
folly::Optional<Tag> getTag(const APCArray* a);

/*
 * Set mode: insert or emplace.
 *
 * Just controls whether we assert about provenance not already being set: we
 * assert for Insert mode, and not for Emplace.
 */
enum class Mode { Insert, Emplace };

/*
 * Set the provenance tag for `a` to `tag`.
 */
template<Mode mode = Mode::Insert> void setTag(ArrayData* a, Tag tag);
template<Mode mode = Mode::Insert> void setTag(const APCArray* a, Tag tag);

/*
 * Clear a tag for a released array---only call this if the array is henceforth
 * unreachable or no longer of a kind that accepts provenance tags
 */
void clearTag(ArrayData* ad);
void clearTag(const APCArray* a);

/*
 * Tag `tv` with provenance for the current PC and unit (if it admits a tag and
 * doesn't already have one).
 *
 * tagTV() takes logical ownership of `tv`, and if it makes any modifications,
 * it will incref the new value and decref the old one.  As such, generally the
 * appropriate use of this function is:
 *
 *    tv = tagTV(tv);
 *
 * without touching the usual TV mutation machinery.
 */
TypedValue tagTV(TypedValue tv);
TypedValue tagTVKnown(TypedValue tv, Tag tag);

/*
 * Produce a static empty array with the given provenance tag.
 *
 * If no tag is provided, we attempt to make one from vmpc(), and failing that
 * we just return the input array.
 */
ArrayData* makeEmptyVec(folly::Optional<Tag> tag = folly::none);
ArrayData* makeEmptyDict(folly::Optional<Tag> tag = folly::none);
ArrayData* tagStaticArr(ArrayData* ad, folly::Optional<Tag> tag = folly::none);

///////////////////////////////////////////////////////////////////////////////

}}

#endif
