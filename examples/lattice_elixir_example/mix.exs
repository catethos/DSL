defmodule LatticeElixirExample.MixProject do
  use Mix.Project

  def project do
    [
      app: :lattice_elixir_example,
      version: "0.1.0",
      elixir: "~> 1.18",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger],
      mod: {LatticeElixirExample.Application, []}
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:rustler, "~> 0.34.0"}
    ]
  end
end
