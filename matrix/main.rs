#![no_std]
#![no_main]


use stm32l4xx_hal::{pac, prelude::*};
use panic_probe as _;
use defmt_rtt as _;
use tp_led_matrix::{Image,matrix::Matrix,RED,BLUE,GREEN};
use dwt_systick_monotonic::DwtSystick;
use dwt_systick_monotonic::ExtU32;
use stm32l4xx_hal::serial::{Config, Event, Rx, Serial};
use heapless::pool::{Box,Pool,Node};
use core::mem::MaybeUninit;

#[rtic::app(device = pac, dispatchers = [USART2, USART3])]
mod app {
    use super::*;

    #[monotonic(binds = SysTick, default = true)]
    type MyMonotonic = DwtSystick<80_000_000>;
    type Instant = <MyMonotonic as rtic::Monotonic>::Instant;

    #[shared]
    struct Shared {
        next_image: Option<Box<Image>>,
        pool: Pool<Image> ,
        changes : u32  
    }

    #[local]
    struct Local {
    matrix: Matrix,
    usart1_rx : Rx<pac::USART1>,
    current_image: Box<Image>,
    rx_image: Box<Image>
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
    defmt::info!("defmt correctly initialized");

    let mut cp = cx.core;
    let dp = cx.device;

    let mut mono = DwtSystick::new(&mut cp.DCB, cp.DWT, cp.SYST, 80_000_000);

    // Initialize the clocks, hardware and matrix using your existing code
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    // Setup the clocks at 80MHz using HSI (by default since HSE/MSI are not configured).
    // The flash wait states will be configured accordingly.
    let clocks = rcc.cfgr.sysclk(80.MHz()).freeze(&mut flash.acr, &mut pwr);

   
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);

    let matrix = Matrix::new(
        gpioa.pa2,
        gpioa.pa3,
        gpioa.pa4,
        gpioa.pa5,
        gpioa.pa6,
        gpioa.pa7,
        gpioa.pa15,
        gpiob.pb0,
        gpiob.pb1,
        gpiob.pb2,
        gpioc.pc3,
        gpioc.pc4,
        gpioc.pc5,
        &mut gpioa.moder,
        &mut gpioa.otyper,
        &mut gpiob.moder,
        &mut gpiob.otyper,
        &mut gpioc.moder,
        &mut gpioc.otyper,
        clocks
    );

    let pb6 = gpiob.pb6.into_alternate::<7>(& mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let pb7 = gpiob.pb7.into_alternate::<7>(& mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

    let config = Config::default().baudrate(38400.bps());

    let mut serial = Serial::usart1(dp.USART1, (pb6,pb7), config, clocks, &mut rcc.apb2);

    serial.listen(Event::Rxne);

    let usart1_rx = serial.split().1;


    let pool: Pool<Image> = Pool::new();
    unsafe {
      static mut MEMORY: MaybeUninit<[Node<Image>; 3]> = MaybeUninit::uninit();
      pool.grow_exact(&mut MEMORY);   // static mut access is unsafe
    }

    let rx_image = pool.alloc().unwrap().init(Image::default());
    let current_image = pool.alloc().unwrap().init(Image::default());

    display::spawn(mono.now()).unwrap();
    
    screensaver::spawn(mono.now(),0).unwrap();
    // Return the resources and the monotonic timer
    (Shared {next_image: None,pool,changes : 0}, Local { matrix, usart1_rx, current_image,rx_image}, init::Monotonics(mono))
    }

    #[task(local = [matrix, current_image ,next_line: usize = 0], shared =[next_image,&pool] ,priority = 2)]
    fn display(mut cx: display::Context,at: Instant) {
    // Display line next_line (cx.local.next_line) of
    // the image (cx.local.image) on the matrix (cx.local.matrix).
    // All those are mutable references.
    cx.local.matrix.send_row(*cx.local.next_line, cx.local.current_image.row(*cx.local.next_line));
    // Increment next_line up to 7 and wraparound to 0
    match *cx.local.next_line {
        7 => {
            cx.shared.next_image.lock(|next_image| {
                    if let Some(mut image) = next_image.take() {
                        core::mem::swap(&mut image, cx.local.current_image);
                        cx.shared.pool.free(image);
                    }
            });
            *cx.local.next_line=0},
        _ => *cx.local.next_line+=1
    }; 
    let next = at + 1.secs() / (60*8);
    display::spawn_at(next,next).unwrap();

}

    #[task(binds = USART1,
        local = [usart1_rx, next_pos: usize = 0 , rx_image],
        shared = [next_image, &pool])]
    fn receive_byte(mut cx: receive_byte::Context)
    {
    let rx_image: &mut Image = cx.local.rx_image;
    let next_pos: &mut usize = cx.local.next_pos;
    if let Ok(b) = cx.local.usart1_rx.read() {
         // Handle the incoming byte according to the SE203 protocol
        // and update next_image
        // Do not forget that next_image.as_mut() might be handy here!
        if b == 0xff{
            *next_pos = 0;
        }
        else if *next_pos < 8*8*3 {
            rx_image.as_mut()[*next_pos] = b;
            *next_pos+=1;
        
            if *next_pos == 8 * 8 * 3 {
                cx.shared.next_image.lock(|next_image| {
                    if let Some(image) = next_image.take(){
                        cx.shared.pool.free(image);
                    }

                let mut future_image = cx.shared.pool.alloc().unwrap().init(Image::default());
                core::mem::swap(rx_image,&mut future_image);

                *next_image = Some(future_image);
                  
                });
                notice_change::spawn().unwrap();
                *next_pos = 0;  
            }
        }
     }
    }

    #[task(shared = [changes])]
    fn notice_change(mut cx: notice_change::Context) {
        cx.shared.changes.lock(|changes| {
            *changes = changes.wrapping_add(1);
        });
    }

    #[task(local = [last_changes: u32 = 0], shared = [next_image, &pool,changes])]
    fn screensaver(mut cx: screensaver::Context, at: Instant, color_index : usize) {
    let current_changes = cx.shared.changes.lock(|changes| *changes);

    if current_changes != *cx.local.last_changes {
        *cx.local.last_changes = current_changes;
    } else {
        let color = match color_index {
            0 => RED,
            1 => GREEN,
            _ => BLUE,
        };
        let gradient_image = cx.shared.pool.alloc().unwrap().init(Image::gradient(color));
        cx.shared.next_image.lock(|next_image| {
            if let Some(previous_image) = next_image.replace(gradient_image) {
                cx.shared.pool.free(previous_image);
            }
        });
    }

    screensaver::spawn_after(1.secs(),at ,(color_index+1)%3).unwrap();
}

    #[idle(local = [])]
    fn idle(_cx: idle::Context) -> ! {
    loop{
    }
}


}