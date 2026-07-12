-- Test VHDL file

library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity test is
  generic (
    flag              : boolean;       -- a flag, can be true or false
      -- a value with a default
    value             : std_logic_vector(15 downto 0) := x"DEAD"
  );
  port (
    clock             : in  std_logic;        -- main clock
    sreset            : in  std_logic := '0'; -- main reset, synchronous, active high

    output_a          : out std_logic;        -- a regular output
      -- one input
    input_b           : in std_logic;
      -- one comment before, that will be ignored because we also have one on the same line as the port
    output_c          : out std_logic;        -- another output
    input_d           : in  std_logic;        -- another input
    data_in           : in  std_logic_vector(15 downto 0);
      -- data_in had no comment
    data_out          : out  std_logic_vector(15 downto 0) -- data out
  );
end entity test;

architecture rtl of test is

    -- enumeration type for the FSM
  type fsm_t is (
    reset, -- reset, initial state
    idle,  -- state when not doing anything 
    read_input, -- check what's on the inputs
    write_output); -- do something on the outputs

    -- a wrapper around the fsm type to test record element access in FSM detection
  type fsm_wrapper_t is record
      -- the actual fsm type
    state : fsm_t;
  end record;

    -- a subtype
  subtype byte_t is std_logic_vector(7 downto 0);

    -- an array
  type matrix_t is array (31 downto 0, 7 downto 0) of byte_t;

    -- state machine signal 
  signal fsm: fsm_wrapper_t;

  signal mysignal : unsigned(15 downto 0) := (others => '0'); -- a signal with a comment on the same line

    -- a constant
  constant c_ones : std_logic_vector(31 downto 0) := (others => '1');

begin  

  process (clock)
  begin
    if rising_edge(clock) then
      if sreset = '1' then
        
        fsm.state <= reset;
        output_a <= '0';
        output_c <= '0';
        data_out <= (others => '0');

      else
        
        case fsm.state is
            -- start and reset state
          when reset =>
              -- out of reset
            fsm.state <= idle;

          when idle => -- normal state when nothing happens
            if input_b = '1' then -- new input
              fsm.state <= read_input;
            end if;

          when read_input => -- waiting for input
            if input_d = '1' then
              fsm.state <= idle;

                -- correct input
            elsif data_in = std_logic_vector(value) then
              fsm.state <= write_output;
            end if;

            -- send output
          when write_output =>
            data_out <= std_logic_vector(value);
            if input_b = '0' then
              fsm.state <= idle;
            else  -- stay
              fsm.state <= write_output;
            end if;

          when others =>
            fsm.state <= reset;
        end case;
      end if;
    end if;
  end process;

  -- an example instantiation
  my_instance : entity work.comp
    generic map (
      enabled => true
    )
    port map (
      clk => clock,
      rst => reset,
      input => input_b,
      output => output_a
    );

end architecture rtl;
