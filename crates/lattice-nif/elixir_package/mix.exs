defmodule Lattice.MixProject do
  use Mix.Project

  @version "0.2.2"
  @source_url "https://github.com/catethos/lattice"

  def project do
    [
      app: :lattice,
      version: @version,
      elixir: "~> 1.14",
      start_permanent: Mix.env() == :prod,
      deps: deps(),

      # Hex.pm metadata
      name: "Lattice",
      description: "Lattice DSL runtime bindings for Elixir with precompiled NIFs",
      package: package(),
      source_url: @source_url
    ]
  end

  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:rustler_precompiled, "~> 0.8"},
      {:rustler, "~> 0.34", optional: true},
      {:ex_doc, "~> 0.31", only: :dev, runtime: false}
    ]
  end

  defp package do
    [
      name: "lattice",
      files: ~w(lib checksum-*.exs mix.exs README.md),
      licenses: ["MIT"],
      links: %{"GitHub" => @source_url}
    ]
  end
end
