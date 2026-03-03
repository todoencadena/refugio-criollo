
use anchor_lang::prelude::*;

// Recuerda poner aquí el Program ID que te genera Anchor al desplegar
declare_id!("EESSL4XMCGCWcWvU227kdhcuDMiPPHt6oqhoxQe9DPue");

#[program]
pub mod refugio_criollo {
    use super::*;

    pub fn crear_refugio(context: Context<NuevoRefugio>, nombre: String) -> Result<()> {
        let fundacion_id = context.accounts.fundacion.key();
        msg!("Fundacion ID: {}", fundacion_id);

        // Vector inicial vacío
        let animales: Vec<Animal> = Vec::new();

        // Asignamos new Refugio al account
        context.accounts.refugio.set_inner(Refugio { 
            fundacion: fundacion_id,
            nombre,
            animales,
        });
        Ok(())
    }

    pub fn registrar_animal(
        context: Context<NuevoAnimal>, 
        nombre: String, 
        edad_meses: u16
    ) -> Result<()> {
        // Validar que quien manda la tx es la fundación dueña
        require!(
            context.accounts.refugio.fundacion == context.accounts.fundacion.key(),
            Errores::NoEresFundacion
        ); 

        let refugio = &mut context.accounts.refugio;

        // Validar no sobrepasar límite de animales
        require!(
            refugio.animales.len() < Refugio::MAX_ANIMALES,
            Errores::AnimalNoExiste // Ideal otro error aquí, pero para ejemplo usamos este
        );

        let animal = Animal {
            nombre,
            edad_meses,
            disponible: true,
        };

        refugio.animales.push(animal);

        Ok(())
    }

    pub fn marcar_adoptado(context: Context<NuevoAnimal>, nombre: String) -> Result<()> {
        require!(
            context.accounts.refugio.fundacion == context.accounts.fundacion.key(),
            Errores::NoEresFundacion
        );

        let animales = &mut context.accounts.refugio.animales;

        for i in 0..animales.len() {
            if animales[i].nombre == nombre {
                animales.remove(i);
                msg!("Animal {} marcado como adoptado!", nombre);
                return Ok(());
            }
        }
        Err(Errores::AnimalNoExiste.into())
    }

    pub fn ver_animales(context: Context<NuevoAnimal>) -> Result<()> {
        require!(
            context.accounts.refugio.fundacion == context.accounts.fundacion.key(),
            Errores::NoEresFundacion
        );

        msg!("Animales en el refugio: {:#?}", context.accounts.refugio.animales);
        Ok(())
    }

    pub fn alternar_disponibilidad(context: Context<NuevoAnimal>, nombre: String) -> Result<()> {
        require!(
            context.accounts.refugio.fundacion == context.accounts.fundacion.key(),
            Errores::NoEresFundacion
        );

        let animales = &mut context.accounts.refugio.animales;
        
        for animal in animales.iter_mut() {
            if animal.nombre == nombre {
                animal.disponible = !animal.disponible;
                msg!(
                    "Animal: {} ahora está {}",
                    nombre,
                    if animal.disponible { "disponible" } else { "no disponible" }
                );
                return Ok(());
            }
        }

        Err(Errores::AnimalNoExiste.into())
    }
}

#[error_code]
pub enum Errores {
    #[msg("Error, no eres la fundación propietaria del refugio")]
    NoEresFundacion,
    #[msg("Error, el animal con el que deseas interactuar no existe")]
    AnimalNoExiste,
}

#[account]
pub struct Refugio {
    fundacion: Pubkey,

    
    nombre: String,

    animales: Vec<Animal>,
}

// Constantes para cálculo de espacio
impl Refugio {
    pub const MAX_ANIMALES: usize = 10;
    pub const MAX_NOMBRE: usize = 60;
    // 32 bytes para Pubkey, 4 bytes prefijo de string + nombre * 4 (capacidad UTF8), 4 prefijo vector + animales tamaño
    pub const INIT_SPACE: usize = 32 
        + 4 + Self::MAX_NOMBRE * 4 
        + 4 + (Self::MAX_ANIMALES * Animal::SIZE);
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct Animal {
    
    nombre: String,

    edad_meses: u16,

    disponible: bool,
}

// Constantes para Animal
impl Animal {
    pub const MAX_NOMBRE: usize = 60;
    // 4 bytes si es string (prefijo) + nombre + 2 bytes u16 + 1 byte bool
    pub const SIZE: usize = 4 + Self::MAX_NOMBRE * 4 + 2 + 1;
}

#[derive(Accounts)]
pub struct NuevoRefugio<'info> {
    #[account(mut)]
    pub fundacion: Signer<'info>,

    #[account(
        init,
        payer = fundacion,
        space = Refugio::INIT_SPACE + 8,
        seeds = [b"refugio", fundacion.key().as_ref()],
        bump,
    )]
    pub refugio: Account<'info, Refugio>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct NuevoAnimal<'info> {
    pub fundacion: Signer<'info>,

    #[account(
        mut,
        seeds = [b"refugio", fundacion.key().as_ref()],
        bump,
    )]
    pub refugio: Account<'info, Refugio>,
}
