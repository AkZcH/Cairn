import { Nav } from "@/components/nav";
import { Hero } from "@/components/hero";
import { StrataStack } from "@/components/strata-stack";
import { BuiltWith } from "@/components/built-with";
import { HowItWorks } from "@/components/how-it-works";
import { Features } from "@/components/features";
import { CTA } from "@/components/cta";
import { Footer } from "@/components/footer";

export default function Home() {
  return (
    <main className="min-h-screen bg-bg text-ink">
      <Nav />
      <Hero />
      <BuiltWith />
      <Features />
      <HowItWorks />
      <CTA />
      <Footer />
    </main>
  );
}
