import Link from "next/link";
import styles from "./page.module.css";

export default function Home() {
  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <span className={styles.brand}>GatheRound</span>
        <nav className={styles.nav}>
          <Link className={styles.btnGhost} href="/login">
            Log in
          </Link>
          <Link className={styles.btnPrimary} href="/signup">
            Sign up
          </Link>
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