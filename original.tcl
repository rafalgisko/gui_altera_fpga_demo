# Adres bazowy UART z Qsys (np. 0x0009_2180)
set UART_ADDR 0x00092180

# Wartość do wysłania - ASCII (np. znak 'A')
set char_to_send 0x41  ;# 'A' w systemie ASCII

# Wysłanie 1 bajtu przez UART
mww $UART_ADDR 0x0 $char_to_send