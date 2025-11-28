# TypeScript Decorators

Dinja supports **TypeScript legacy decorators** (`experimentalDecorators`) which are used by most popular frameworks including Angular, NestJS, TypeORM, and MobX.

## Supported Decorator Types

### Class Decorators

```typescript
@Injectable()
class UserService {
  // ...
}

@Controller('/users')
class UserController {
  // ...
}
```

### Method Decorators

```typescript
class UserController {
  @Get('/:id')
  @UseGuards(AuthGuard)
  async getUser(@Param('id') id: string) {
    // ...
  }
}
```

### Property Decorators

```typescript
class User {
  @Column({ type: 'varchar', length: 255 })
  name: string;

  @Column({ type: 'int' })
  age: number;
}
```

### Parameter Decorators

```typescript
class UserController {
  constructor(
    @Inject('USER_SERVICE') private userService: UserService,
  ) {}

  getUser(@Param('id') id: string, @Query('include') include?: string) {
    // ...
  }
}
```

### Accessor Decorators

```typescript
class Point {
  private _x: number;

  @configurable(false)
  get x() {
    return this._x;
  }
}
```

## Framework Examples

### Angular

```typescript
@Component({
  selector: 'app-user-profile',
  templateUrl: './user-profile.component.html',
  styleUrls: ['./user-profile.component.css']
})
export class UserProfileComponent implements OnInit {
  @Input() userId: string;
  @Output() userUpdated = new EventEmitter<User>();

  @ViewChild('form') form: NgForm;

  ngOnInit() {
    // ...
  }
}
```

### NestJS

```typescript
@Controller('cats')
export class CatsController {
  constructor(private catsService: CatsService) {}

  @Get()
  @UseInterceptors(LoggingInterceptor)
  async findAll(): Promise<Cat[]> {
    return this.catsService.findAll();
  }

  @Post()
  @HttpCode(201)
  async create(@Body() createCatDto: CreateCatDto) {
    return this.catsService.create(createCatDto);
  }
}
```

### TypeORM

```typescript
@Entity('users')
export class User {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ unique: true })
  email: string;

  @Column({ select: false })
  password: string;

  @CreateDateColumn()
  createdAt: Date;

  @ManyToOne(() => Organization, org => org.users)
  @JoinColumn({ name: 'organization_id' })
  organization: Organization;
}
```

### MobX

```typescript
class TodoStore {
  @observable
  todos: Todo[] = [];

  @observable
  filter: string = '';

  @computed
  get filteredTodos() {
    return this.todos.filter(todo =>
      todo.title.includes(this.filter)
    );
  }

  @action
  addTodo(title: string) {
    this.todos.push({ title, completed: false });
  }
}
```

### class-validator

```typescript
class CreateUserDto {
  @IsEmail()
  @IsNotEmpty()
  email: string;

  @IsString()
  @MinLength(8)
  @MaxLength(100)
  password: string;

  @IsOptional()
  @IsInt()
  @Min(0)
  @Max(150)
  age?: number;
}
```

## Decorator Metadata

Dinja enables `emitDecoratorMetadata`, which allows frameworks to access type information at runtime via `reflect-metadata`. This is required for:

- Dependency injection (Angular, NestJS, inversify)
- Validation (class-validator, class-transformer)
- ORM mapping (TypeORM, TypeGraphQL)

Example of how metadata is used:

```typescript
import 'reflect-metadata';

function Injectable(): ClassDecorator {
  return (target) => {
    // Get constructor parameter types
    const paramTypes = Reflect.getMetadata('design:paramtypes', target);
    console.log('Dependencies:', paramTypes);
  };
}
```

## Limitations

### TC39 Stage 3 Decorators Not Supported

The new [TC39 Stage 3 decorator proposal](https://github.com/tc39/proposal-decorators) (2023 standard) uses different syntax and semantics:

```typescript
// TC39 Stage 3 syntax - NOT YET SUPPORTED
class Foo {
  @logged
  accessor x = 1;  // accessor keyword is TC39-specific
}
```

This is tracked in [oxc-project/oxc#9170](https://github.com/oxc-project/oxc/issues/9170). The parser can parse TC39 decorator syntax, but transformation to JavaScript is not yet implemented.

**Workaround**: Continue using legacy decorators. Most frameworks (Angular, NestJS, TypeORM, etc.) still use legacy decorators and will for the foreseeable future.

### tsconfig.json Settings

When using decorators in your TypeScript project, ensure your `tsconfig.json` includes:

```json
{
  "compilerOptions": {
    "experimentalDecorators": true,
    "emitDecoratorMetadata": true
  }
}
```

This ensures TypeScript itself handles the decorators correctly during development, while Dinja handles the transformation at render time.
