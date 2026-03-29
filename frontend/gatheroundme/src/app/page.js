"use client";

import { useState, useEffect } from "react";
import styles from "./page.module.css";

/** * ZITADEL AUTH HELPERS 
 */
function b64url(buf) {
  return btoa(String.fromCharCode(...new Uint8Array(buf)))
    .replace(/\+/g, "-")
    .replace(/\//g, "_")
    .replace(/=/g, "");
}

async function pkce() {
  const v = b64url(crypto.getRandomValues(new Uint8Array(32)));
  const d = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(v));
  return { verifier: v, challenge: b64url(d) };
}

function randState() {
  return b64url(crypto.getRandomValues(new Uint8Array(16)));
}

const SK = { v: "z_verifier", s: "z_state", cfg: "z_cfg" };

export default function Home() {
  const [config, setConfig] = useState({
    domain: "",
    clientId: "",
    scope: "openid profile email",
  });

  // Fetch Zitadel Config from Backend (Mocked for now)
  useEffect(() => {
    async function fetchConfig() {
      try {
        /* // Once backend is ready, this performs the GET call 
        const response = await fetch('/api/auth-config'); 
        const data = await response.json();
        setConfig({
          domain: data.domain,
          clientId: data.clientId,
          scope: "openid profile email"
        });
        */

        // Placeholder data for manual testing
        setConfig({
          domain: "https://your-instance.zitadel.cloud",
          clientId: "your-client-id-here",
          scope: "openid profile email",
        });
      } catch (err) {
        console.error("Failed to fetch auth config", err);
      }
    }
    fetchConfig();
  }, []);

  const handleLogin = async () => {
    if (!config.domain || !config.clientId) {
      alert("Auth configuration is missing. Check backend setup.");
      return;
    }

    const cleanDomain = config.domain.replace(/\/$/, "");
    const { verifier, challenge } = await pkce();
    const state = randState();

    // Store config and PKCE values in session to verify upon return
    sessionStorage.setItem(SK.cfg, JSON.stringify({ ...config, domain: cleanDomain }));
    sessionStorage.setItem(SK.v, verifier);
    sessionStorage.setItem(SK.s, state);

    // Redirect to activities page after Zitadel auth
    const redirectUri = window.location.origin + "/activities";

    const u = new URL(`${cleanDomain}/oauth/v2/authorize`);
    u.searchParams.set("response_type", "code");
    u.searchParams.set("client_id", config.clientId);
    u.searchParams.set("redirect_uri", redirectUri);
    u.searchParams.set("scope", config.scope);
    u.searchParams.set("state", state);
    u.searchParams.set("code_challenge", challenge);
    u.searchParams.set("code_challenge_method", "S256");

    window.location.href = u.toString();
  };

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <span className={styles.brand}>
          <img
            src="/campfirelogo512.png"
            alt="campfire logo"
            style={{
              width: "1.75rem",
              height: "1.75rem",
              objectFit: "contain",
              verticalAlign: "middle",
              marginRight: "0.35rem",
            }}
          />
          GatheRound
        </span>
        <nav className={styles.nav}>
          {/* Replaced Link with Button to trigger handleLogin logic */}
          <button 
            className={styles.btnPrimary} 
            onClick={handleLogin}
            style={{ cursor: 'pointer', border: 'none' }}
          >
            Login/Signup
          </button>
        </nav>
      </header>

      <main className={styles.main}>
        <section className={styles.hero}>
          <p className={styles.eyebrow}>Anti-social media, pro-real life</p>
          <h1 className={styles.title}>
            Post what you want to do. Find people who want to do it with you.
          </h1>
          <p className={styles.lede}>
            Share activities you actually want to show up for - not endless feeds.
            Others can RSVP and meet you in real life, not just in the comments.
          </p>
        </section>

        <section className={styles.how} aria-labelledby="how-heading">
          <h2 id="how-heading" className={styles.sectionTitle}>
            How it works
          </h2>
          <ol className={styles.steps}>
            <li className={styles.step}>
              <span className={styles.stepNum}>1</span>
              <div>
                <h3 className={styles.stepTitle}>Tell us about yourself</h3>
                <p className={styles.stepText}>
                  Write a short paragraph about yourself and the kinds of activities
                  you care about. This helps us suggest user-posted activities that align with your interests.
                </p>
              </div>
            </li>
            <li className={styles.step}>
              <span className={styles.stepNum}>2</span>
              <div>
                <h3 className={styles.stepTitle}>Post real activities</h3>
                <p className={styles.stepText}>
                  Share something you want to do with others: a hike, a game night,
                  even an extremely niche activity. No algorithmic noise, just intent.
                </p>
              </div>
            </li>
            <li className={styles.step}>
              <span className={styles.stepNum}>3</span>
              <div>
                <h3 className={styles.stepTitle}>RSVP &amp; show up</h3>
                <p className={styles.stepText}>
                  People who have similar interests can RSVP and meet up with you in real life. You
                  coordinate and meet offline.
                </p>
              </div>
            </li>
          </ol>
        </section>
      </main>

      <footer className={styles.footer}>
        <p>© {new Date().getFullYear()} GatheRound</p>
      </footer>
    </div>
  );
}