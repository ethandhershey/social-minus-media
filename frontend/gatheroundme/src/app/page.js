import Link from "next/link";
import styles from "./page.module.css";

export default function Home() {
  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <span className={styles.brand}>Gatheround</span>
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
          <p className={styles.eyebrow}>Anti–social media, pro–real life</p>
          <h1 className={styles.title}>
            Post what you want to do. Find people who want to do it with you.
          </h1>
          <p className={styles.lede}>
            Share activities you actually want to show up for—not endless feeds.
            Others can RSVP and meet you in the world, not just in the comments.
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
                <h3 className={styles.stepTitle}>Tell us your vibe</h3>
                <p className={styles.stepText}>
                  Write a short paragraph about you and the kinds of activities
                  you care about. That becomes the signal for who you match with.
                </p>
              </div>
            </li>
            <li className={styles.step}>
              <span className={styles.stepNum}>2</span>
              <div>
                <h3 className={styles.stepTitle}>Post real activities</h3>
                <p className={styles.stepText}>
                  Share something you want to do with others—a walk, a game night,
                  a hike. No algorithmic noise, just intent.
                </p>
              </div>
            </li>
            <li className={styles.step}>
              <span className={styles.stepNum}>3</span>
              <div>
                <h3 className={styles.stepTitle}>RSVP &amp; show up</h3>
                <p className={styles.stepText}>
                  People who resonate with your profile can join your plans. You
                  coordinate and meet offline.
                </p>
              </div>
            </li>
          </ol>
        </section>
      </main>
      <footer className={styles.footer}>
        <p>© {new Date().getFullYear()} Gatheround</p>
      </footer>
    </div>
  );
}