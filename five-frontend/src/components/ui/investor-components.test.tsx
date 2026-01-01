import React from "react";
import { render, screen } from "@testing-library/react";
import {
  Section,
  Card,
  MetricCard,
  UseCaseCard,
  ComparisonItem,
  MoatCard,
  TokenBenefit,
} from "./investor-components";
import { Zap } from "lucide-react";

describe("Investor Components", () => {
  describe("Section", () => {
    it("renders section with title and subtitle", () => {
      render(
        <Section
          title="Test Section"
          subtitle="Test subtitle"
          icon={<Zap />}
          color="iris"
        >
          <div>Content</div>
        </Section>
      );

      expect(screen.getByText("Test Section")).toBeInTheDocument();
      expect(screen.getByText("Test subtitle")).toBeInTheDocument();
      expect(screen.getByText("Content")).toBeInTheDocument();
    });

    it("applies correct color class", () => {
      const { container } = render(
        <Section
          title="Test"
          subtitle="Test"
          icon={<Zap />}
          color="gold"
        >
          <div>Content</div>
        </Section>
      );

      const header = container.querySelector("h2");
      expect(header).toHaveClass("text-rose-pine-gold");
    });

    it("respects align prop", () => {
      const { container } = render(
        <Section
          title="Test"
          subtitle="Test"
          icon={<Zap />}
          align="right"
        >
          <div>Content</div>
        </Section>
      );

      const flex = container.querySelector("[class*='flex']");
      expect(flex).toHaveClass("items-end", "text-right");
    });
  });

  describe("Card", () => {
    it("renders card with title and description", () => {
      render(
        <Card
          title="Feature"
          description="This is a feature"
          icon={<Zap />}
        />
      );

      expect(screen.getByText("Feature")).toBeInTheDocument();
      expect(screen.getByText("This is a feature")).toBeInTheDocument();
    });

    it("applies hover animation", () => {
      const { container } = render(
        <Card
          title="Feature"
          description="This is a feature"
          icon={<Zap />}
        />
      );

      const div = container.querySelector("[class*='rounded-3xl']");
      expect(div).toHaveClass("hover:shadow-rose-pine-iris/10");
    });
  });

  describe("MetricCard", () => {
    it("renders metric with label and value", () => {
      render(
        <MetricCard
          label="Efficiency"
          value="800x"
          description="Smaller bytecode"
        />
      );

      expect(screen.getByText("Efficiency")).toBeInTheDocument();
      expect(screen.getByText("800x")).toBeInTheDocument();
      expect(screen.getByText("Smaller bytecode")).toBeInTheDocument();
    });

    it("applies gold color styling", () => {
      const { container } = render(
        <MetricCard
          label="Efficiency"
          value="800x"
          description="Smaller bytecode"
        />
      );

      const value = screen.getByText("800x");
      expect(value).toHaveClass("text-rose-pine-gold");
    });
  });

  describe("UseCaseCard", () => {
    it("renders use case with title and description", () => {
      render(
        <UseCaseCard
          title="NFTs"
          icon={<Zap />}
          description="Dynamic NFT updates"
          color="love"
        />
      );

      expect(screen.getByText("NFTs")).toBeInTheDocument();
      expect(screen.getByText("Dynamic NFT updates")).toBeInTheDocument();
    });

    it("applies correct color variant", () => {
      const { container } = render(
        <UseCaseCard
          title="NFTs"
          icon={<Zap />}
          description="Dynamic NFT updates"
          color="gold"
        />
      );

      const header = container.querySelector("h3");
      expect(header).toHaveClass("text-rose-pine-gold");
    });
  });

  describe("ComparisonItem", () => {
    it("renders comparison with traditional and five options", () => {
      render(
        <ComparisonItem
          metric="Cost"
          traditional="$126"
          five="$0.002"
        />
      );

      expect(screen.getByText("Cost")).toBeInTheDocument();
      expect(screen.getByText("$126")).toBeInTheDocument();
      expect(screen.getByText("$0.002")).toBeInTheDocument();
    });

    it("highlights five option with foam color", () => {
      render(
        <ComparisonItem
          metric="Cost"
          traditional="$126"
          five="$0.002"
        />
      );

      const fiveOption = screen.getByText("$0.002");
      expect(fiveOption).toHaveClass("text-rose-pine-foam");
    });
  });

  describe("MoatCard", () => {
    it("renders moat advantage with title and description", () => {
      render(
        <MoatCard
          title="Ecosystem Lock-in"
          description="Components that don't exist elsewhere"
        />
      );

      expect(screen.getByText("Ecosystem Lock-in")).toBeInTheDocument();
      expect(
        screen.getByText("Components that don't exist elsewhere")
      ).toBeInTheDocument();
    });

    it("applies love color styling", () => {
      const { container } = render(
        <MoatCard
          title="Ecosystem Lock-in"
          description="Components that don't exist elsewhere"
        />
      );

      const header = container.querySelector("h3");
      expect(header).toHaveClass("text-rose-pine-love");
    });
  });

  describe("TokenBenefit", () => {
    it("renders benefit with icon and text", () => {
      render(<TokenBenefit icon="→" text="Governance voting" />);

      expect(screen.getByText("→")).toBeInTheDocument();
      expect(screen.getByText("Governance voting")).toBeInTheDocument();
    });

    it("applies gold icon color", () => {
      const { container } = render(
        <TokenBenefit icon="→" text="Governance voting" />
      );

      const icon = container.querySelector("span");
      expect(icon).toHaveClass("text-rose-pine-gold");
    });
  });

  describe("Component Memoization", () => {
    it("Section is memoized", () => {
      expect(Section.$$typeof).toBeDefined();
    });

    it("Card is memoized", () => {
      expect(Card.$$typeof).toBeDefined();
    });

    it("MetricCard is memoized", () => {
      expect(MetricCard.$$typeof).toBeDefined();
    });

    it("UseCaseCard is memoized", () => {
      expect(UseCaseCard.$$typeof).toBeDefined();
    });

    it("ComparisonItem is memoized", () => {
      expect(ComparisonItem.$$typeof).toBeDefined();
    });

    it("MoatCard is memoized", () => {
      expect(MoatCard.$$typeof).toBeDefined();
    });

    it("TokenBenefit is memoized", () => {
      expect(TokenBenefit.$$typeof).toBeDefined();
    });
  });
});
