-- Test VHDL file

library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

package test_pkg is
    
    -- a constant in the package
    constant c_package_constant : std_logic_vector(31 downto 0) := x"deadbeef";

    -- an enumeration type without comments on each element
    type test_status_t is ( working, not_working);
    
end package;
