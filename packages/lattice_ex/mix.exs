defmodule Lattice.MixProject do
  use Mix.Project

  @version "0.1.0"
  @source_url "https://github.com/catethos/DSL"

  def project do
    [
      app: :lattice,
      version: @version,
      elixir: "~> 1.14",
      start_permanent: Mix.env() == :prod,
      deps: deps(),
      package: package(),
      description: "Lattice DSL runtime for Elixir - a domain-specific language with LLM integration",

      # Docs
      name: "Lattice",
      source_url: @source_url,
      docs: docs()
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      # For precompiled NIFs (production)
      {:rustler_precompiled, "~> 0.7"},
      # For local development/compilation (optional)
      {:rustler, "~> 0.34.0", optional: true},
      # Documentation
      {:ex_doc, "~> 0.31", only: :dev, runtime: false}
    ]
  end

  defp package do
    [
      name: "lattice",
      files: ~w(lib priv .formatter.exs mix.exs README.md LICENSE checksum-*.exs),
      licenses: ["MIT"],
      links: %{"GitHub" => @source_url}
    ]
  end

  defp docs do
    [
      main: "readme",
      extras: ["README.md"]
    ]
  end
end
