"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import styles from "./activities.module.css";

const DESC_PREVIEW_CHARS = 100;

const INITIAL_ACTIVITIES = [
  {
    id: "1",
    title: "Saturday morning coffee & walk",
    location: "Riverside Park, north entrance",
    eventDate: "2026-04-04T10:00",
    description: "Casual coffee to-go and a slow lap around the loop.",
    distanceMiles: 1.2,
    alignmentScore: 0.91,
  },
];

function emptyRsvpMap(activities) {
  return Object.fromEntries(activities.map((a) => [a.id, null]));
}

function formatEventDate(dateStr) {
  if (!dateStr) return "";
  const date = new Date(dateStr);
  return date.toLocaleDateString("en-US", {
    weekday: "short",
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
  });
}

export default function ActivitiesPage() {
  const router = useRouter();
  const [activities, setActivities] = useState([]);
  const [isLoading, setIsLoading] = useState(true);
  const [query, setQuery] = useState("");
  const [rsvpById, setRsvpById] = useState({});

  // Filters and Modals
  const [maxDistanceMiles, setMaxDistanceMiles] = useState(25);
  const [minAlignment, setMinAlignment] = useState(0.5);
  const [minMatch, setMinMatch] = useState(0.5); // Default to 50%
  const [filterOpen, setFilterOpen] = useState(false);
  const [addOpen, setAddOpen] = useState(false);
  const [newTitle, setNewTitle] = useState("");
  const [newLocation, setNewLocation] = useState("");
  const [newDate, setNewDate] = useState("");
  const [newDescription, setNewDescription] = useState("");
  

  const filterRef = useRef(null);

  useEffect(() => {
    async function initializeAuthAndData() {
      const params = new URLSearchParams(window.location.search);
      const code = params.get("code");
      const state = params.get("state");
      const baseUrl = process.env.NEXT_PUBLIC_API_URL || "";

      let token = sessionStorage.getItem("access_token");

      try {
        // 1. HANDLE OAUTH CALLBACK (If code exists in URL)
        if (code) {
          const storedState = sessionStorage.getItem("z_state");
          const verifier = sessionStorage.getItem("z_verifier");
          const config = JSON.parse(sessionStorage.getItem("z_cfg") || "{}");

          if (state !== storedState) throw new Error("State mismatch");

          const tokenRes = await fetch(`${config.domain}/oauth/v2/token`, {
            method: "POST",
            headers: { "Content-Type": "application/x-www-form-urlencoded" },
            body: new URLSearchParams({
              grant_type: "authorization_code",
              code,
              redirect_uri: window.location.origin + "/activities",
              client_id: config.clientId,
              code_verifier: verifier,
            }),
          });

          const tokenData = await tokenRes.json();
          if (tokenData.access_token) {
            token = tokenData.access_token;
            sessionStorage.setItem("access_token", token);
            // Clean URL without triggering a reload
            window.history.replaceState({}, document.title, "/activities");
          }
        }
        
        console.log(token);
        // 2. VERIFY TOKEN PRESENCE (Logic: If no token, go to landing)
        if (!token) {
          router.push("/");
          return;
        }
        else
        {
          console.log("Yes token exists");
        }

        // 3. CHECK PROFILE
        const profileRes = await fetch(`${baseUrl}/api/profile`, {
          headers: { Authorization: `Bearer ${token}` },
        });

        // Check if the response is actually JSON before parsing
        const contentType = profileRes.headers.get("content-type");
        if (!contentType || !contentType.includes("application/json")) {
          const text = await profileRes.text();
          console.error("Backend returned non-JSON response:", text);
          throw new Error("Server returned HTML instead of JSON. Check your API URL.");
        }

        if (profileRes.status === 401 || profileRes.status === 403) {
          sessionStorage.removeItem("access_token");
          router.push("/");
          return;
        }

        if (!profileRes.ok) throw new Error("Profile fetch failed");

        const userData = await profileRes.json();

        // If user has NO city, force /signup
        if (!userData.city) {
          router.push("/signup");
          return;
        }

        // 4. LOAD ACTIVITIES
      const activitiesRes = await fetch(`${baseUrl}/api/events`, {
        headers: { Authorization: `Bearer ${token}` },
      });

      if (activitiesRes.ok) {
        const data = await activitiesRes.json();
        
        // Map through data to add a random alignment score
        const activitiesWithScores = data.map(event => ({
          ...event,
          // Generates a random score between 0.50 and 0.99
          alignmentScore: parseFloat((Math.random() * (0.99 - 0.5) + 0.5).toFixed(2))
        }));

        // Sort by alignmentScore descending (highest at top)
        const sortedActivities = activitiesWithScores.sort((a, b) => b.alignmentScore - a.alignmentScore);

        setActivities(sortedActivities);
        setRsvpById(emptyRsvpMap(sortedActivities));
      }
      } catch (err) {
        console.error("Initialization error:", err);
        // Only redirect on critical failures
        // router.push("/"); 
      } finally {
        setIsLoading(false);
      }
    }

    initializeAuthAndData();
  }, [router]);

  // --- Handlers & Helpers ---
  const closeAdd = useCallback(() => {
    setAddOpen(false);
    setNewTitle("");
    setNewLocation("");
    setNewDate("");
    setNewDescription("");
  }, []);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    return (activities || []).filter((a) => {
      // Distance Filter
      if (a.distanceMiles > maxDistanceMiles) return false;
      
      // Match Score Filter (New)
      if (a.alignmentScore < minMatch) return false;
      
      // Search Query Filter
      if (!q) return true;
      const blob = `${a.title} ${a.address} ${a.description} ${a.address}`.toLowerCase();
      return blob.includes(q);
    });
  }, [activities, query, maxDistanceMiles, minMatch]); // Added minMatch to dependency array

  function submitRsvp(activityId, response) {
    setRsvpById((prev) => ({ ...prev, [activityId]: response }));
  }

  async function submitNewActivity(e) {
    e.preventDefault();
    if (!newTitle || !newLocation || !newDate || !newDescription) return;

    const token = sessionStorage.getItem("access_token");
    const baseUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";

    setIsLoading(true);

    try {
      // 1. Get current GPS for the event location
      const getCoords = () => new Promise((res, rej) => {
        navigator.geolocation.getCurrentPosition(res, rej, { timeout: 5000 });
      });
      
      let lat = null;
      let lng = null;
      try {
        const pos = await getCoords();
        lat = pos.coords.latitude;
        lng = pos.coords.longitude;
      } catch (geoErr) {
        console.warn("Could not get GPS for event, posting without coords.");
      }

      // 2. Format payload to match Rust CreateEventBody struct
      const payload = {
        title: newTitle.trim(),
        description: newDescription.trim(),
        address: newLocation.trim(), // Mapping frontend "location" to backend "address"
        latitude: lat,
        longitude: lng,
        // Backend expects RFC3339 string (e.g., "2026-04-04T10:00:00Z")
        start_time: new Date(newDate).toISOString(),
        max_capacity: null, 
      };

      // 3. POST to /api/events (MATCHING THE RUST ROUTE)
      const response = await fetch(`${baseUrl}/api/events`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Authorization": `Bearer ${token}`,
        },
        body: JSON.stringify(payload),
      });

      if (!response.ok) {
        const errText = await response.text();
        throw new Error(errText || `Server error: ${response.status}`);
      }

      const createdEvent = await response.json();

      // Update UI
      setActivities((prev) => [createdEvent, ...prev]);
      setRsvpById((prev) => ({ ...prev, [createdEvent.id]: null }));
      closeAdd();
      alert("Activity posted!");

    } catch (err) {
      console.error("Post error:", err);
      alert(err.message);
    } finally {
      setIsLoading(false);
    }
  }

  // --- UI Components ---
  if (isLoading) {
    return <div className={styles.page}><main className={styles.main}><p className={styles.empty}>Verifying session...</p></main></div>;
  }

  return (
    <div className={styles.page}>
      <header className={styles.topBar}>
        <div className={styles.topLead}>
          <div className={styles.brand}>
            <Link className={styles.brandLink} href="/">
              <img src="/campfirelogo512.PNG" alt="logo" style={{ width: "1.75rem", marginRight: "0.35rem" }} />
              GatheRound
            </Link>
          </div>
          <button type="button" className={styles.addActivityBtn} onClick={() => setAddOpen(true)}>
            Add activity
          </button>
        </div>
        <div className={styles.search}>
          <input
            className={styles.searchInput}
            type="search"
            placeholder="Search…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </div>
        <div className={styles.actions}>
          <div className={styles.filterWrap} ref={filterRef}>
            <button className={styles.iconBtn} onClick={() => setFilterOpen(!filterOpen)}>
              <FilterIcon />
            </button>
            {filterOpen && (
              <div className={styles.filterPanel}>
                <div className={styles.filterField}>
                  <p className={styles.filterPanelTitle}>Distance</p>
                  <select className={styles.filterSelect} value={maxDistanceMiles} onChange={(e) => setMaxDistanceMiles(Number(e.target.value))}>
                    <option value="5">5 mi</option>
                    <option value="25">25 mi</option>
                    <option value="50">50 mi</option>
                  </select>
                </div>
                {/* New Min Match Filter */}
                <div className={styles.filterField}>
  <div style={{ overflow: 'hidden' }}>
    <span className={styles.filterPanelTitle}>Min Match</span>
    <span className={styles.filterScoreValue}>{Math.round(minMatch * 100)}%</span>
  </div>
  <input 
    type="range"
    className={styles.filterRange}
    min="0"
    max="1"
    step="0.05"
    value={minMatch}
    onChange={(e) => setMinMatch(parseFloat(e.target.value))}
  />
  <p className={styles.filterHint}>Showing events that match your interests.</p>
</div>
              </div>
            )}
          </div>
          <Link className={styles.iconBtn} href="/profile"><ProfileIcon /></Link>
        </div>
      </header>

      {addOpen && (
        <div className={styles.modalRoot}>
          <div className={styles.modalBackdrop} onClick={closeAdd} />
          <div className={styles.modal} role="dialog">
            <div className={styles.modalHeader}>
              <h2 className={styles.modalTitle}>New activity</h2>
              <button className={styles.modalClose} onClick={closeAdd}><XIcon /></button>
            </div>
            <form className={styles.modalForm} onSubmit={submitNewActivity}>
              <input className={styles.modalInput} placeholder="Name" value={newTitle} onChange={(e) => setNewTitle(e.target.value)} required />
              <input className={styles.modalInput} placeholder="Location" value={newLocation} onChange={(e) => setNewLocation(e.target.value)} required />
              <input className={styles.modalInput} type="datetime-local" value={newDate} onChange={(e) => setNewDate(e.target.value)} required />
              <textarea className={styles.modalTextarea} placeholder="Description" value={newDescription} onChange={(e) => setNewDescription(e.target.value)} required />
              <div className={styles.modalActions}>
                <button type="button" onClick={closeAdd}>Cancel</button>
                <button type="submit" className={styles.modalBtnPrimary}>Post</button>
              </div>
            </form>
          </div>
        </div>
      )}

      <main className={styles.main}>
        {filtered.length === 0 ? (
          <p className={styles.empty}>No activities found.</p>
        ) : (
          <ul className={styles.list}>
            {filtered.map((a) => (
            <li key={a.id}>
              <article className={styles.card}>
      <div className={styles.cardHeader} style={{ display: 'flex', justifyContent: 'space-between' }}>
        <h2 className={styles.cardTitle}>{a.title}</h2>
        {/* Display the random alignment score */}
        <span className={styles.scoreBadge} style={{ color: '#7dae8a', fontWeight: 'bold' }}>
          {Math.round(a.alignmentScore * 100)}% Match
        </span>
      </div>
      
      <p className={styles.cardLocation}>{a.address || "No location specified"}</p>
      <p className={styles.cardDate}>{formatEventDate(a.start_time)}</p>
      
      <CardDescription 
        description={a.description} 
        previewChars={DESC_PREVIEW_CHARS} 
        styles={styles} 
      />
                
                <div className={styles.cardFooter}>
                  <div className={styles.rsvpChoices}>
                    <button 
                      className={`${styles.rsvpIconBtn} ${rsvpById[a.id] === 'going' ? styles.rsvpGoingSelected : ''}`}
                      onClick={() => submitRsvp(a.id, "going")}
                    >
                      <CheckIcon />
                    </button>
                    <button 
                      className={`${styles.rsvpIconBtn} ${rsvpById[a.id] === 'not_going' ? styles.rsvpNotSelected : ''}`}
                      onClick={() => submitRsvp(a.id, "not_going")}
                    >
                      <XIcon />
                    </button>
                  </div>
                </div>
              </article>
            </li>
          ))}
          </ul>
        )}
      </main>
    </div>
  );
}

// Sub-components and Icons
function CardDescription({ description, previewChars, styles: s }) {
  const [expanded, setExpanded] = useState(false);
  const needsMore = description.length > previewChars;
  const preview = needsMore && !expanded ? `${description.slice(0, previewChars).trim()}…` : description;
  return (
    <div className={s.cardDesc}>
      <p className={s.cardDescText}>{preview}</p>
      {needsMore && (
        <button className={s.showMoreBtn} onClick={() => setExpanded(!expanded)}>
          {expanded ? "Show less" : "Show more"}
        </button>
      )}
    </div>
  );
}

function FilterIcon() { return <svg width="20" height="20" viewBox="0 0 24 24"><path fill="currentColor" d="M10 18h4v-2h-4v2zM3 6v2h18V6H3zm3 7h12v-2H6v2z" /></svg>; }
function ProfileIcon() { return <svg width="20" height="20" viewBox="0 0 24 24"><path fill="currentColor" d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z" /></svg>; }
function CheckIcon() { return <svg width="20" height="20" viewBox="0 0 24 24"><path fill="currentColor" d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z" /></svg>; }
function XIcon() { return <svg width="20" height="20" viewBox="0 0 24 24"><path fill="currentColor" d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12 19 6.41z" /></svg>; }