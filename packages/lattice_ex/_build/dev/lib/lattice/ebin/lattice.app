{application,lattice,
             [{modules,['Elixir.Lattice','Elixir.Lattice.Native']},
              {optional_applications,[rustler]},
              {applications,[kernel,stdlib,elixir,logger,rustler_precompiled,
                             rustler]},
              {description,"Lattice DSL runtime for Elixir - a domain-specific language with LLM integration"},
              {registered,[]},
              {vsn,"0.1.2"}]}.
