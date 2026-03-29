"use client";

import Link from "next/link";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import styles from "./activities.module.css";

const DESC_PREVIEW_CHARS = 100;

const INITIAL_ACTIVITIES = [
  {
    id: "1",
    title: "Saturday morning coffee & walk",
    location: "Riverside Park, north entrance",
    description:
      "Casual coffee to-go and a slow lap around the loop. Small group; we mostly chat about nothing serious. No pressure to stay the whole time—drop in when you can.",
    distanceMiles: 1.2,
    alignmentScore: 0.91,
  },
  {
    id: "2",
    title: "Board games at the library",
    location: "Main St. Public Library, community room",
    description: "Medium-weight games; beginners welcome.",
    distanceMiles: 4.8,
    alignmentScore: 0.72,
  },
  {
    id: "3",
    title: "Easy hike — under 3 miles",
    location: "Trailhead: Quarry Rd lot",
    description:
      "Early-ish start, steady pace, lots of breaks. Bringing snacks to share at the overlook if the weather cooperates.",
    distanceMiles: 12.0,
    alignmentScore: 0.65,
  },
];

function emptyRsvpMap(activities) {
  return Object.fromEntries(activities.map((a) => [a.id, null]));
}

export default function ActivitiesPage() {
  const [activities, setActivities] = useState(INITIAL_ACTIVITIES);
  const [query, setQuery] = useState("");
  const [maxDistanceMiles, setMaxDistanceMiles] = useState(25);
  const [minAlignment, setMinAlignment] = useState(0.5);
  const [filterOpen, setFilterOpen] = useState(false);
  const [rsvpById, setRsvpById] = useState(() => emptyRsvpMap(INITIAL_ACTIVITIES));

  const [addOpen, setAddOpen] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newLocation, setNewLocation] = useState("");
  const [newDescription, setNewDescription] = useState("");

  const filterRef = useRef(null);

  const closeAdd = useCallback(() => {
    setAddOpen(false);
    setNewTitle("");
    setNewLocation("");
    setNewDescription("");
  }, []);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return activities.filter((a) => {
      if (a.distanceMiles > maxDistanceMiles) return false;
      if (a.alignmentScore < minAlignment) return false;
      if (!q) return true;
      const blob = `${a.title} ${a.location} ${a.description}`.toLowerCase();
      return blob.includes(q);
    });
  }, [activities, query, maxDistanceMiles, minAlignment]);

  const closeFilter = useCallback(() => setFilterOpen(false), []);

  useEffect(() => {
    if (!filterOpen) return;
    function onKey(e) {
      if (e.key === "Escape") closeFilter();
    }
    function onPointer(e) {
      if (filterRef.current && !filterRef.current.contains(e.target)) {
        closeFilter();
      }
    }
    document.addEventListener("keydown", onKey);
    document.addEventListener("mousedown", onPointer);
    document.addEventListener("touchstart", onPointer, { passive: true });
    return () => {
      document.removeEventListener("keydown", onKey);
      document.removeEventListener("mousedown", onPointer);
      document.removeEventListener("touchstart", onPointer);
    };
  }, [filterOpen, closeFilter]);

  useEffect(() => {
    if (!addOpen) return;
    function onKey(e) {
      if (e.key === "Escape") closeAdd();
    }
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [addOpen, closeAdd]);

  function submitRsvp(activityId, response) {
    setRsvpById((prev) => ({ ...prev, [activityId]: response }));
    // await fetch(`/api/activities/${activityId}/rsvp`, {
    //   method: "POST",
    //   headers: { "Content-Type": "application/json" },
    //   body: JSON.stringify({ response }),
    // });
  }

  function submitNewActivity(e) {
    e.preventDefault();
    const title = newTitle.trim();
    const location = newLocation.trim();
    const description = newDescription.trim();
    if (!title || !location || !description) return;

    const id =
      typeof crypto !== "undefined" && crypto.randomUUID
        ? crypto.randomUUID()
        : `local-${Date.now()}`;

    // POST /api/activities
    // Content-Type: application/json
    // Body: { title, location, description }
    // Optional: lat/lng if you geocode client-side or collect from a map picker.
    // Response: created row including id, distanceMiles, alignmentScore from server.
    // await fetch("/api/activities", {
    //   method: "POST",
    //   headers: { "Content-Type": "application/json" },
    //   body: JSON.stringify({ title, location, description }),
    // });

    const optimistic = {
      id,
      title,
      location,
      description,
      distanceMiles: 0,
      alignmentScore: 1,
    };
    setActivities((prev) => [optimistic, ...prev]);
    setRsvpById((prev) => ({ ...prev, [id]: null }));
    closeAdd();
  }

  return (
    <div className={styles.page}>
      <header className={styles.topBar}>
        <div className={styles.topLead}>
          <div className={styles.brand}>
            <Link className={styles.brandLink} href="/">
              GatheRound
            </Link>
          </div>
          <button
            type="button"
            className={styles.addActivityBtn}
            onClick={() => setAddOpen(true)}
          >
            <span className={styles.addActivityLong}>Add activity</span>
            <span className={styles.addActivityShort} aria-hidden>
              Add
            </span>
          </button>
        </div>
        <div className={styles.search}>
          <label htmlFor="activity-search" className="sr-only">
            Search activities
          </label>
          <input
            id="activity-search"
            className={styles.searchInput}
            type="search"
            placeholder="Search…"
            enterKeyHint="search"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            autoComplete="off"
          />
        </div>
        <div className={styles.actions}>
          <div className={styles.filterWrap} ref={filterRef}>
            <button
              type="button"
              className={styles.iconBtn}
              aria-expanded={filterOpen}
              aria-haspopup="true"
              aria-controls="activity-filters"
              onClick={() => setFilterOpen((o) => !o)}
              title="Filters"
            >
              <FilterIcon />
            </button>
            {filterOpen ? (
              <div
                id="activity-filters"
                className={styles.filterPanel}
                role="region"
                aria-label="Search filters"
              >
                <div className={styles.filterField}>
                  <p className={styles.filterPanelTitle}>Distance</p>
                  <label className={styles.filterLabel} htmlFor="filter-distance">
                    Max distance (miles)
                  </label>
                  <select
                    id="filter-distance"
                    className={styles.filterSelect}
                    value={String(maxDistanceMiles)}
                    onChange={(e) =>
                      setMaxDistanceMiles(Number(e.target.value))
                    }
                  >
                    <option value="5">5 mi</option>
                    <option value="10">10 mi</option>
                    <option value="25">25 mi</option>
                    <option value="50">50 mi</option>
                  </select>
                  <p className={styles.filterHint}>
                    Placeholder until user location is wired up.
                  </p>
                </div>
                <div className={styles.filterField}>
                  <p className={styles.filterPanelTitle}>Alignment</p>
                  <label
                    className={styles.filterLabel}
                    htmlFor="filter-alignment"
                  >
                    Minimum alignment score
                  </label>
                  <select
                    id="filter-alignment"
                    className={styles.filterSelect}
                    value={String(minAlignment)}
                    onChange={(e) => setMinAlignment(Number(e.target.value))}
                  >
                    <option value="0.5">0.5+</option>
                    <option value="0.6">0.6+</option>
                    <option value="0.7">0.7+</option>
                    <option value="0.8">0.8+</option>
                    <option value="0.9">0.9+</option>
                  </select>
                </div>
              </div>
            ) : null}
          </div>
          <Link
            className={styles.iconBtn}
            href="/profile"
            aria-label="Profile"
            title="Profile"
          >
            <ProfileIcon />
          </Link>
        </div>
      </header>

      {addOpen ? (
        <div className={styles.modalRoot}>
          <button
            type="button"
            className={styles.modalBackdrop}
            aria-label="Close dialog"
            onClick={closeAdd}
          />
          <div
            className={styles.modal}
            role="dialog"
            aria-modal="true"
            aria-labelledby="add-activity-title"
          >
            <div className={styles.modalHeader}>
              <h2 id="add-activity-title" className={styles.modalTitle}>
                New activity
              </h2>
              <button
                type="button"
                className={styles.modalClose}
                onClick={closeAdd}
                aria-label="Close"
              >
                <XIcon />
              </button>
            </div>
            <form className={styles.modalForm} onSubmit={submitNewActivity}>
              <div className={styles.modalField}>
                <label className={styles.modalLabel} htmlFor="new-activity-title">
                  Name
                </label>
                <input
                  id="new-activity-title"
                  className={styles.modalInput}
                  value={newTitle}
                  onChange={(e) => setNewTitle(e.target.value)}
                  required
                  autoComplete="off"
                />
              </div>
              <div className={styles.modalField}>
                <label className={styles.modalLabel} htmlFor="new-activity-location">
                  Location
                </label>
                <input
                  id="new-activity-location"
                  className={styles.modalInput}
                  value={newLocation}
                  onChange={(e) => setNewLocation(e.target.value)}
                  required
                  autoComplete="street-address"
                />
              </div>
              <div className={styles.modalField}>
                <label
                  className={styles.modalLabel}
                  htmlFor="new-activity-description"
                >
                  Short description
                </label>
                <textarea
                  id="new-activity-description"
                  className={styles.modalTextarea}
                  rows={4}
                  value={newDescription}
                  onChange={(e) => setNewDescription(e.target.value)}
                  required
                />
              </div>
              <div className={styles.modalActions}>
                <button
                  type="button"
                  className={styles.modalBtnGhost}
                  onClick={closeAdd}
                >
                  Cancel
                </button>
                <button type="submit" className={styles.modalBtnPrimary}>
                  Post
                </button>
              </div>
            </form>
          </div>
        </div>
      ) : null}

      <main className={styles.main}>
        {filtered.length === 0 ? (
          <p className={styles.empty}>No activities match your filters.</p>
        ) : (
          <ul className={styles.list}>
            {filtered.map((a) => {
              const choice = rsvpById[a.id];
              return (
                <li key={a.id}>
                  <article className={styles.card}>
                    <h2 className={styles.cardTitle}>{a.title}</h2>
                    <p className={styles.cardLocation}>{a.location}</p>
                    <div className={styles.meta}>
                      <span>
                        <strong>{a.distanceMiles.toFixed(1)}</strong> mi away
                      </span>
                      <span>
                        Alignment:{" "}
                        <strong>{a.alignmentScore.toFixed(2)}</strong>
                      </span>
                    </div>
                    <CardDescription
                      description={a.description}
                      previewChars={DESC_PREVIEW_CHARS}
                      styles={styles}
                    />
                    <div className={styles.cardFooter}>
                      <span
                        className={styles.rsvpLabel}
                        id={`rsvp-${a.id}-label`}
                      >
                        RSVP
                      </span>
                      <div
                        className={styles.rsvpChoices}
                        role="group"
                        aria-labelledby={`rsvp-${a.id}-label`}
                      >
                        <button
                          type="button"
                          className={`${styles.rsvpIconBtn} ${
                            choice === "going" ? styles.rsvpGoingSelected : ""
                          }`}
                          onClick={() => submitRsvp(a.id, "going")}
                          aria-pressed={choice === "going"}
                          aria-label="Going"
                        >
                          <CheckIcon />
                        </button>
                        <button
                          type="button"
                          className={`${styles.rsvpIconBtn} ${
                            choice === "not_going"
                              ? styles.rsvpNotSelected
                              : ""
                          }`}
                          onClick={() => submitRsvp(a.id, "not_going")}
                          aria-pressed={choice === "not_going"}
                          aria-label="Not going"
                        >
                          <XIcon />
                        </button>
                      </div>
                    </div>
                  </article>
                </li>
              );
            })}
          </ul>
        )}
      </main>
    </div>
  );
}

function CardDescription({ description, previewChars, styles: s }) {
  const [expanded, setExpanded] = useState(false);
  const needsMore = description.length > previewChars;
  const preview =
    needsMore && !expanded
      ? `${description.slice(0, previewChars).trim()}…`
      : description;

  return (
    <div className={s.cardDesc}>
      <p className={s.cardDescText}>{preview}</p>
      {needsMore ? (
        <button
          type="button"
          className={s.showMoreBtn}
          onClick={() => setExpanded((v) => !v)}
          aria-expanded={expanded}
        >
          {expanded ? "Show less" : "Show more"}
        </button>
      ) : null}
    </div>
  );
}

function FilterIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" aria-hidden="true">
      <path
        fill="currentColor"
        d="M10 18h4v-2h-4v2zM3 6v2h18V6H3zm3 7h12v-2H6v2z"
      />
    </svg>
  );
}

function ProfileIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" aria-hidden="true">
      <path
        fill="currentColor"
        d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"
      />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" aria-hidden="true">
      <path
        fill="currentColor"
        d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"
      />
    </svg>
  );
}

function XIcon() {
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" aria-hidden="true">
      <path
        fill="currentColor"
        d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12 19 6.41z"
      />
    </svg>
  );
}