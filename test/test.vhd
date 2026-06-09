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

  type fsm_t is (reset, idle, read_input, write_output);

  type fsm_wrapper_t is record
    state : fsm_t;
  end record;

    -- state machine signal 
  signal fsm: fsm_wrapper_t;

  signal mysignal : unsigned(15 downto 0) := (others => '0'); -- a signal with a comment on the same line

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
end architecture rtl;
